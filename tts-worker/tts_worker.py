#!/usr/bin/env python3
"""TTS Worker — generates speech audio from text using Kokoro, Chatterbox, or Qwen3-TTS."""

import sys
import json
import os


def _kokoro_models_dir():
    """Return the directory where Kokoro ONNX models are cached."""
    cache = os.path.expanduser("~/.cache/speech-to-text/models/kokoro-onnx")
    os.makedirs(cache, exist_ok=True)
    return cache


def _download_file(url, dest):
    """Download a file with progress to stderr."""
    import urllib.request
    print(f"Telechargement: {os.path.basename(dest)}...", file=sys.stderr)
    urllib.request.urlretrieve(url, dest)
    print(f"Termine: {os.path.basename(dest)}", file=sys.stderr)


def _download_kokoro_models():
    """Download Kokoro ONNX model and voices if not already cached."""
    models_dir = _kokoro_models_dir()
    model_path = os.path.join(models_dir, "kokoro-v1.0.onnx")
    voices_path = os.path.join(models_dir, "voices-v1.0.bin")

    base_url = "https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.0"

    if not os.path.exists(model_path):
        _download_file(f"{base_url}/kokoro-v1.0.onnx", model_path)

    if not os.path.exists(voices_path):
        _download_file(f"{base_url}/voices-v1.0.bin", voices_path)

    return model_path, voices_path


def run_kokoro(text, language, output_path, voice=None, speed=None, blend_voice=None, blend_ratio=None):
    """Kokoro TTS — lightweight ONNX model, fast, CPU-friendly.

    Args:
        voice: Voice code (e.g. "af_heart"). Auto-selected from language if None.
        speed: Speed multiplier 0.5-2.0. Default 1.0.
        blend_voice: Second voice code for blending. None = no blend.
        blend_ratio: Tuple (weight1, weight2) for blending. E.g. (0.7, 0.3).
    """
    import soundfile as sf
    import numpy as np
    from kokoro_onnx import Kokoro

    model_path, voices_path = _download_kokoro_models()
    kokoro = Kokoro(model_path, voices_path)

    # Voice selection: explicit or auto from language
    if voice is None:
        voice_map = {"fr": "ff_siwis", "en": "af_heart"}
        voice = voice_map.get(language, "af_heart")

    # Speed
    if speed is None:
        speed = 1.0

    # Language code for phonemizer
    lang_map = {"fr": "fr-fr", "en": "en-us"}
    lang_code = lang_map.get(language, "en-us")

    # Build voice style (with optional blending)
    voice_style = kokoro.get_voice_style(voice)
    if blend_voice and blend_ratio:
        voice_style_2 = kokoro.get_voice_style(blend_voice)
        w1, w2 = blend_ratio
        voice_style = w1 * voice_style + w2 * voice_style_2

    samples, sample_rate = kokoro.create(text, voice=voice_style, speed=speed, lang=lang_code)

    if samples is None or len(samples) == 0:
        raise RuntimeError("Kokoro n'a genere aucun audio")

    sf.write(output_path, samples, sample_rate)


def run_chatterbox(text, language, output_path, use_gpu):
    """Chatterbox TTS — high quality, 23 languages, paralinguistic tags."""
    import torch
    import torchaudio
    from chatterbox.tts import ChatterboxTTS

    device = "cuda" if use_gpu and torch.cuda.is_available() else "cpu"
    model = ChatterboxTTS.from_pretrained(device=device)
    wav = model.generate(text)
    torchaudio.save(output_path, wav, model.sr)


def run_qwen3_tts(text, language, output_path, use_gpu):
    """Qwen3-TTS — multilingual with excellent French, controllable prosody."""
    import torch
    import soundfile as sf
    from transformers import AutoTokenizer, AutoModelForSpeechSeq2Seq, pipeline

    device = "cuda:0" if use_gpu and torch.cuda.is_available() else "cpu"
    torch_dtype = torch.float16 if use_gpu and torch.cuda.is_available() else torch.float32

    model_id = "Qwen/Qwen3-TTS"

    synthesiser = pipeline(
        "text-to-speech",
        model=model_id,
        device=device,
        torch_dtype=torch_dtype,
    )

    # Language instruction prefix for Qwen3-TTS
    lang_instruction = {
        "fr": "[French]",
        "en": "[English]",
    }
    prefix = lang_instruction.get(language, "")
    full_text = f"{prefix} {text}" if prefix else text

    speech = synthesiser(full_text)
    sf.write(output_path, speech["audio"][0], speech["sampling_rate"])


def main():
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue

        try:
            req = json.loads(line)
            engine = req["engine"]
            text = req["text"]
            language = req.get("language", "en")
            output_path = req["output_path"]
            use_gpu = req.get("use_gpu", False)

            if engine == "kokoro":
                kokoro_voice = req.get("voice")
                kokoro_speed = req.get("speed")
                kokoro_blend_voice = req.get("blend_voice")
                kokoro_blend_ratio = req.get("blend_ratio")
                if kokoro_blend_ratio:
                    kokoro_blend_ratio = tuple(kokoro_blend_ratio)
                run_kokoro(
                    text, language, output_path,
                    voice=kokoro_voice,
                    speed=kokoro_speed,
                    blend_voice=kokoro_blend_voice,
                    blend_ratio=kokoro_blend_ratio,
                )
            elif engine == "chatterbox":
                run_chatterbox(text, language, output_path, use_gpu)
            elif engine == "qwen3":
                run_qwen3_tts(text, language, output_path, use_gpu)
            else:
                raise ValueError(f"Moteur inconnu: {engine}")

            resp = {"success": True, "audio_path": output_path}

        except Exception as e:
            import traceback
            traceback.print_exc(file=sys.stderr)
            resp = {"success": False, "error": str(e)}

        print(json.dumps(resp), flush=True)


if __name__ == "__main__":
    main()
