import os
import sys
import re
import asyncio
import io
import shutil
import edge_tts
from fastapi import FastAPI, Query, HTTPException
from fastapi.responses import Response
import uvicorn

# Force UTF-8 encoding on Windows
if sys.platform == "win32":
    sys.stdin = io.TextIOWrapper(sys.stdin.buffer, encoding='utf-8')
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(SCRIPT_DIR, "../../.."))

# Ensure ffmpeg and scripts are in PATH
_py_dir = os.path.dirname(sys.executable)
_py_scripts = os.path.join(_py_dir, "Scripts")
for p in [_py_dir, _py_scripts]:
    if os.path.exists(p) and p not in os.environ.get("PATH", ""):
        os.environ["PATH"] = p + os.pathsep + os.environ.get("PATH", "")

# Warm global RVC loader
_rvc_loader = None
YAE_MIKO_PTH = os.path.join(PROJECT_ROOT, "data", "models", "rvc", "yaemiko", "Yae_MikoJP.pth")

def init_rvc():
    global _rvc_loader
    if _rvc_loader is None:
        try:
            from infer_rvc_python import BaseLoader
            hubert_local = os.path.join(PROJECT_ROOT, "data", "models", "rvc", "hubert")
            hubert_arg = hubert_local if os.path.exists(hubert_local) else None
            _rvc_loader = BaseLoader(only_cpu=False, hubert_path=hubert_arg)
            if os.path.exists(YAE_MIKO_PTH):
                _rvc_loader.apply_conf(
                    tag="yaemiko",
                    file_model=YAE_MIKO_PTH,
                    file_index="",
                    pitch_algo="pm",
                    pitch_lvl=0,
                    index_influence=0.0
                )
                print(f"[RVC Daemon] Yae Miko pre-loaded into GPU VRAM successfully!", file=sys.stderr)
            else:
                print(f"[RVC Daemon] Model file not found: {YAE_MIKO_PTH}", file=sys.stderr)
        except Exception as e:
            print(f"[RVC Daemon] Failed to initialize RVC: {e}", file=sys.stderr)
            _rvc_loader = False
    return _rvc_loader

from contextlib import asynccontextmanager
import uuid

# In-memory RAM cache for repeated or common phrases (0ms instant return)
_mem_cache = {}
_rvc_lock = asyncio.Lock()

def is_japanese(text: str) -> bool:
    return bool(re.search(r'[\u3040-\u309f\u30a0-\u30ff]', text))

async def synthesize_base_tts(text: str, base_voice: str, out_path: str):
    try:
        # rate=+6% provides energetic, natural anime VTuber cadence and speeds up RVC synthesis by 20%
        communicate = edge_tts.Communicate(text, base_voice, rate="+6%", pitch="-3Hz")
        await communicate.save(out_path)
    except Exception as e:
        # Retry with standard default prosody if pitch shift unsupported
        print(f"[Edge-TTS] Retrying with default prosody due to: {e}", file=sys.stderr)
        communicate = edge_tts.Communicate(text, base_voice)
        await communicate.save(out_path)

def convert_rvc(input_path: str, output_path: str) -> bool:
    loader = init_rvc()
    if not loader:
        return False
    try:
        result = loader(input_path, "yaemiko")
        if isinstance(result, (list, tuple)) and len(result) > 0:
            result = result[0]
        if result and os.path.exists(result):
            shutil.copy2(result, output_path)
            return True
    except Exception as e:
        print(f"[RVC Daemon] Conversion error: {e}", file=sys.stderr)
        return False
    return False

@asynccontextmanager
async def lifespan(app: FastAPI):
    # Warm up RVC on daemon launch
    print("[RVC Daemon] Starting up and warming up GPU...", file=sys.stderr)
    init_rvc()
    print("[RVC Daemon] Ready to serve instant voice requests on port 5005!", file=sys.stderr)
    yield

app = FastAPI(title="VirtualCharacter Yae Miko RVC Voice Daemon", lifespan=lifespan)

@app.get("/health")
def health():
    return {"status": "ok", "voice": "yaemiko", "gpu_warm": _rvc_loader is not None and _rvc_loader is not False}

@app.get("/tts")
async def tts(text: str = Query(..., min_length=1)):
    clean = re.sub(r'\*[^*]+\*', '', text).strip()
    clean = clean.replace('"', '').replace('“', '').replace('”', '').strip()
    if not clean:
        clean = text

    # Check RAM cache for 0ms instant playback
    if clean in _mem_cache:
        return Response(content=_mem_cache[clean], media_type="audio/mpeg", headers={
            "Cache-Control": "public, max-age=86400",
            "Access-Control-Allow-Origin": "*"
        })

    base_voice = "ja-JP-NanamiNeural" if is_japanese(clean) else "vi-VN-HoaiMyNeural"

    os.makedirs(os.path.join(PROJECT_ROOT, "data"), exist_ok=True)
    temp_id = uuid.uuid4().hex[:12]
    temp_base = os.path.join(PROJECT_ROOT, "data", f"daemon_base_{temp_id}.mp3")
    temp_out = os.path.join(PROJECT_ROOT, "data", f"daemon_out_{temp_id}.mp3")

    try:
        await synthesize_base_tts(clean, base_voice, temp_base)
        
        # Serialize RVC GPU inference to prevent concurrent CUDA contention
        async with _rvc_lock:
            converted = await asyncio.to_thread(convert_rvc, temp_base, temp_out)

        if not converted:
            raise HTTPException(status_code=500, detail="RVC conversion failed")

        with open(temp_out, "rb") as f:
            audio_data = f.read()

        # Cache in RAM (limit to 100 entries)
        if len(_mem_cache) > 100:
            _mem_cache.pop(next(iter(_mem_cache)))
        _mem_cache[clean] = audio_data

        return Response(content=audio_data, media_type="audio/mpeg", headers={
            "Cache-Control": "public, max-age=86400",
            "Access-Control-Allow-Origin": "*"
        })
    finally:
        for p in [temp_base, temp_out]:
            if os.path.exists(p):
                try:
                    os.remove(p)
                except Exception:
                    pass

if __name__ == "__main__":
    uvicorn.run(app, host="127.0.0.1", port=5005, log_level="warning")
