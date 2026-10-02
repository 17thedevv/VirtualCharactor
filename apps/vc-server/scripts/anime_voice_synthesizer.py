import sys
import os
import re
import asyncio
import edge_tts

# Force UTF-8 encoding on Windows
if sys.platform == "win32":
    import io
    sys.stdin = io.TextIOWrapper(sys.stdin.buffer, encoding='utf-8')
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(SCRIPT_DIR, "../../.."))

# Ensure ffmpeg and Python scripts are in system PATH
_py_dir = os.path.dirname(sys.executable)
_py_scripts = os.path.join(_py_dir, "Scripts")
for p in [_py_dir, _py_scripts]:
    if os.path.exists(p) and p not in os.environ.get("PATH", ""):
        os.environ["PATH"] = p + os.pathsep + os.environ.get("PATH", "")

# Global RVC loader instance (lazy loaded)
_rvc_loader = None
_rvc_configured_tag = None

def get_rvc_loader():
    global _rvc_loader
    if _rvc_loader is None:
        try:
            from infer_rvc_python import BaseLoader
            hubert_local = os.path.join(PROJECT_ROOT, "data", "models", "rvc", "hubert")
            hubert_arg = hubert_local if os.path.exists(hubert_local) else None
            _rvc_loader = BaseLoader(only_cpu=False, hubert_path=hubert_arg)
            print(f"[RVC] BaseLoader initialized successfully (hubert: {hubert_arg})", file=sys.stderr)
        except Exception as e:
            print(f"[RVC] Could not initialize BaseLoader: {e}", file=sys.stderr)
            _rvc_loader = False
    return _rvc_loader if _rvc_loader is not False else None

def is_japanese(text: str) -> bool:
    """Detect if text contains Japanese hiragana or katakana characters"""
    return bool(re.search(r'[\u3040-\u309f\u30a0-\u30ff]', text))

async def synthesize_base_tts(text: str, base_voice: str, out_path: str):
    """Generate base phonetic vocal guidance via Edge-TTS"""
    # Tune prosody to match anime Onee-san gentle cadence
    communicate = edge_tts.Communicate(
        text,
        base_voice,
        rate="-2%",
        pitch="-5Hz"
    )
    await communicate.save(out_path)

YAE_MIKO_PTH = os.path.join(PROJECT_ROOT, "data", "models", "rvc", "yaemiko", "Yae_MikoJP.pth")

def apply_rvc_conversion(input_audio_path: str, output_audio_path: str, _model_tag: str = "yaemiko") -> bool:
    """Transform base vocal into Onee-san Yae Miko voice using RVC"""
    loader = get_rvc_loader()
    if not loader:
        return False

    if not os.path.exists(YAE_MIKO_PTH):
        print(f"[RVC] Yae Miko model file not found at: {YAE_MIKO_PTH}", file=sys.stderr)
        return False

    try:
        global _rvc_configured_tag
        if _rvc_configured_tag != "yaemiko":
            loader.apply_conf(
                tag="yaemiko",
                file_model=YAE_MIKO_PTH,
                file_index="",
                pitch_algo="pm",
                pitch_lvl=0,
                index_influence=0.0
            )
            _rvc_configured_tag = "yaemiko"

        # Convert
        result = loader(input_audio_path, "yaemiko")
        if isinstance(result, (list, tuple)) and len(result) > 0:
            result = result[0]
        if result and os.path.exists(result):
            import shutil
            shutil.copy2(result, output_audio_path)
            print(f"[RVC] Voice conversion successful (yaemiko)! Saved: {output_audio_path}", file=sys.stderr)
            return True
    except Exception as e:
        print(f"[RVC] Voice conversion error (yaemiko): {e}", file=sys.stderr)
        return False

    return False

async def main(text: str, output_path: str, model_tag: str = "yaemiko"):
    # Clean text of markdown action markers
    clean = re.sub(r'\*[^*]+\*', '', text).strip()
    if not clean:
        clean = text

    # Select base voice: Japanese seiyuu for Japanese, Vietnamese female for Vietnamese
    if is_japanese(clean):
        base_voice = "ja-JP-NanamiNeural"
    else:
        base_voice = "vi-VN-HoaiMyNeural"

    temp_base = output_path + ".base.mp3"
    try:
        await synthesize_base_tts(clean, base_voice, temp_base)

        # Apply RVC voice conversion
        converted = apply_rvc_conversion(temp_base, output_path, model_tag)
        if not converted:
            raise RuntimeError(f"RVC voice conversion failed for model '{model_tag}'. Check model files.")
    finally:
        if os.path.exists(temp_base):
            try:
                os.remove(temp_base)
            except:
                pass

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python anime_voice_synthesizer.py <text> <output_path> [model_tag]", file=sys.stderr)
        sys.exit(1)

    text_arg = sys.argv[1]
    out_arg = sys.argv[2]
    tag_arg = sys.argv[3] if len(sys.argv) > 3 else "yaemiko"

    asyncio.run(main(text_arg, out_arg, tag_arg))
