#!/usr/bin/env python3
"""
Lightweight CPU Whisper Transcription Service for VirtualCharacter.
Executes on Intel i5-12500H CPU using CTranslate2 int8 quantization.
Keeps GPU 0 MB VRAM free for LLM / VLM inference.
"""

import sys
import json
import argparse
from faster_whisper import WhisperModel

_MODEL_CACHE = {}

def get_model(model_size: str = "tiny"):
    if model_size not in _MODEL_CACHE:
        _MODEL_CACHE[model_size] = WhisperModel(
            model_size,
            device="cpu",
            compute_type="int8",
            num_workers=2
        )
    return _MODEL_CACHE[model_size]

def transcribe(audio_path: str, model_size: str = "tiny", language: str = None):
    model = get_model(model_size)
    segments, info = model.transcribe(
        audio_path,
        beam_size=3,
        language=language,
        vad_filter=True,
    )
    full_text = " ".join(seg.text for seg in segments).strip()
    return {
        "success": True,
        "text": full_text,
        "language": info.language,
        "probability": float(info.language_probability),
    }

def main():
    parser = argparse.ArgumentParser(description="VirtualCharacter Whisper Transcriber")
    parser.add_argument("audio_path", help="Path to input WAV/MP3 file")
    parser.add_argument("--model", default="tiny", help="Whisper model size (tiny, base)")
    parser.add_argument("--language", default=None, help="Explicit language code (e.g. en, vi, ja)")
    args = parser.parse_args()

    try:
        res = transcribe(args.audio_path, args.model, args.language)
        print(json.dumps(res, ensure_ascii=False))
    except Exception as e:
        print(json.dumps({"success": False, "error": str(e)}))
        sys.exit(1)

if __name__ == "__main__":
    main()
