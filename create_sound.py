"""
Utility to generate a placeholder alert sound.
This should be replaced with an actual alert sound.
"""
import numpy as np
import wave
import os

def create_placeholder_sound(output_path, duration=2.0, frequency=440, sample_rate=44100):
    """Create a placeholder alert sound (simple beep)."""
    # Generate time array
    t = np.linspace(0, duration, int(sample_rate * duration))
    
    # Generate sine wave with varying frequency (alarm-like)
    wave_data = np.sin(2 * np.pi * frequency * t) + np.sin(2 * np.pi * (frequency * 1.5) * t)
    
    # Add some amplitude modulation to make it more attention-grabbing
    modulation = np.sin(2 * np.pi * 5 * t)  # 5 Hz modulation
    wave_data = wave_data * (0.5 + 0.5 * modulation)
    
    # Normalize to 16-bit range
    wave_data = (wave_data * 32767 / np.max(np.abs(wave_data))).astype(np.int16)
    
    # Write to WAV file
    with wave.open(output_path, 'w') as wav_file:
        wav_file.setnchannels(1)  # Mono
        wav_file.setsampwidth(2)  # 2 bytes per sample (16-bit)
        wav_file.setframerate(sample_rate)
        wav_file.writeframes(wave_data.tobytes())
    
    print(f"Created placeholder sound at {output_path}")

if __name__ == "__main__":
    # Create assets directory if it doesn't exist
    os.makedirs("assets/sounds", exist_ok=True)
    
    # Create placeholder sound
    create_placeholder_sound("assets/sounds/alert.wav")
