/// デフォルトサウンドデータ（440Hz A4音 0.3秒 WAVフォーマット）
///
/// WAVファイル構造:
/// - サンプルレート: 44100 Hz
/// - ビット深度: 16bit
/// - チャンネル: モノラル
/// - 周波数: 440Hz (A4)
/// - 長さ: 約0.3秒 (13230サンプル)
///
/// サウンドには自然なフェードイン・フェードアウトが適用されています。
pub const DEFAULT_SOUND_DATA: &[u8] = &generate_beep_wav();

/// コンパイル時に440Hz WAVデータを生成
const fn generate_beep_wav() -> [u8; WAV_FILE_SIZE] {
    let mut data = [0u8; WAV_FILE_SIZE];

    // RIFF header
    data[0] = b'R';
    data[1] = b'I';
    data[2] = b'F';
    data[3] = b'F';

    // File size - 8 (little endian)
    let file_size_minus_8 = (WAV_FILE_SIZE - 8) as u32;
    data[4] = (file_size_minus_8 & 0xFF) as u8;
    data[5] = ((file_size_minus_8 >> 8) & 0xFF) as u8;
    data[6] = ((file_size_minus_8 >> 16) & 0xFF) as u8;
    data[7] = ((file_size_minus_8 >> 24) & 0xFF) as u8;

    // WAVE format
    data[8] = b'W';
    data[9] = b'A';
    data[10] = b'V';
    data[11] = b'E';

    // fmt subchunk
    data[12] = b'f';
    data[13] = b'm';
    data[14] = b't';
    data[15] = b' ';

    // Subchunk1Size (16 for PCM)
    data[16] = 16;
    data[17] = 0;
    data[18] = 0;
    data[19] = 0;

    // AudioFormat (1 = PCM)
    data[20] = 1;
    data[21] = 0;

    // NumChannels (1 = mono)
    data[22] = 1;
    data[23] = 0;

    // SampleRate (44100)
    data[24] = (SAMPLE_RATE & 0xFF) as u8;
    data[25] = ((SAMPLE_RATE >> 8) & 0xFF) as u8;
    data[26] = ((SAMPLE_RATE >> 16) & 0xFF) as u8;
    data[27] = ((SAMPLE_RATE >> 24) & 0xFF) as u8;

    // ByteRate = SampleRate * NumChannels * BitsPerSample/8 = 44100 * 1 * 2 = 88200
    let byte_rate = SAMPLE_RATE * 2;
    data[28] = (byte_rate & 0xFF) as u8;
    data[29] = ((byte_rate >> 8) & 0xFF) as u8;
    data[30] = ((byte_rate >> 16) & 0xFF) as u8;
    data[31] = ((byte_rate >> 24) & 0xFF) as u8;

    // BlockAlign (NumChannels * BitsPerSample/8)
    data[32] = 2; // 1 * 16/8 = 2
    data[33] = 0;

    // BitsPerSample (16)
    data[34] = 16;
    data[35] = 0;

    // data subchunk
    data[36] = b'd';
    data[37] = b'a';
    data[38] = b't';
    data[39] = b'a';

    // Subchunk2Size (NumSamples * NumChannels * BitsPerSample/8)
    let data_size = (NUM_SAMPLES * 2) as u32;
    data[40] = (data_size & 0xFF) as u8;
    data[41] = ((data_size >> 8) & 0xFF) as u8;
    data[42] = ((data_size >> 16) & 0xFF) as u8;
    data[43] = ((data_size >> 24) & 0xFF) as u8;

    // Generate 440Hz sine wave samples with fade in/out
    let mut i = 0;
    while i < NUM_SAMPLES {
        // Calculate sine value using Taylor series approximation
        // sin(x) ≈ x - x³/6 + x⁵/120 - x⁷/5040
        let phase = (i as i64 * FREQUENCY as i64 * 2 * 31416) / (SAMPLE_RATE as i64 * 10000);
        let phase_normalized = phase % 62832; // 2π * 10000
        let x = if phase_normalized > 31416 {
            phase_normalized - 62832
        } else {
            phase_normalized
        };

        // Convert to smaller range for calculation (-π to π as fixed point)
        let x_fixed = x as i32; // x is in range -31416 to 31416 (represents -π to π * 10000)

        // sin approximation using Taylor series (fixed point, scaled by 10000)
        let x2 = (x_fixed as i64 * x_fixed as i64) / 100_000_000; // x²
        let x3 = (x2 * x_fixed as i64) / 10000; // x³
        let x5 = (x3 * x2) / 10000; // x⁵
        let x7 = (x5 * x2) / 10000; // x⁷

        let sin_val = x_fixed as i64 - x3 / 6 + x5 / 120 - x7 / 5040;

        // Apply envelope (fade in/out)
        let envelope = if i < FADE_SAMPLES {
            // Fade in
            (i as i64 * 10000) / FADE_SAMPLES as i64
        } else if i >= NUM_SAMPLES - FADE_SAMPLES {
            // Fade out
            ((NUM_SAMPLES - i) as i64 * 10000) / FADE_SAMPLES as i64
        } else {
            10000 // Full volume
        };

        // Calculate final sample value
        // sin_val is scaled by 10000, envelope is scaled by 10000
        // We want 16-bit signed (-32768 to 32767)
        // Use 70% volume to avoid clipping
        let sample = ((sin_val * envelope * 7) / 1_000_000_000) as i16;

        // Write as little-endian 16-bit
        let sample_bytes = sample.to_le_bytes();
        let offset = WAV_HEADER_SIZE + i * 2;
        data[offset] = sample_bytes[0];
        data[offset + 1] = sample_bytes[1];

        i += 1;
    }

    data
}

// Constants for WAV generation
const SAMPLE_RATE: u32 = 44100;
const FREQUENCY: u32 = 440; // A4 note
const DURATION_MS: u32 = 300; // 0.3 seconds
const NUM_SAMPLES: usize = (SAMPLE_RATE * DURATION_MS / 1000) as usize; // 13230 samples
const FADE_SAMPLES: usize = NUM_SAMPLES / 10; // 10% fade in/out
const WAV_HEADER_SIZE: usize = 44;
const WAV_FILE_SIZE: usize = WAV_HEADER_SIZE + NUM_SAMPLES * 2; // header + 16bit samples

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_sound_data_not_empty() {
        assert!(!DEFAULT_SOUND_DATA.is_empty());
        assert_eq!(DEFAULT_SOUND_DATA.len(), WAV_FILE_SIZE);
    }

    #[test]
    fn test_wav_header_valid() {
        // Check RIFF header
        assert_eq!(&DEFAULT_SOUND_DATA[0..4], b"RIFF");
        assert_eq!(&DEFAULT_SOUND_DATA[8..12], b"WAVE");
        assert_eq!(&DEFAULT_SOUND_DATA[12..16], b"fmt ");
        assert_eq!(&DEFAULT_SOUND_DATA[36..40], b"data");
    }

    #[test]
    fn test_wav_format_pcm() {
        // Audio format should be 1 (PCM)
        let audio_format = u16::from_le_bytes([DEFAULT_SOUND_DATA[20], DEFAULT_SOUND_DATA[21]]);
        assert_eq!(audio_format, 1);
    }

    #[test]
    fn test_wav_channels_mono() {
        let num_channels = u16::from_le_bytes([DEFAULT_SOUND_DATA[22], DEFAULT_SOUND_DATA[23]]);
        assert_eq!(num_channels, 1);
    }

    #[test]
    fn test_wav_sample_rate() {
        let sample_rate = u32::from_le_bytes([
            DEFAULT_SOUND_DATA[24],
            DEFAULT_SOUND_DATA[25],
            DEFAULT_SOUND_DATA[26],
            DEFAULT_SOUND_DATA[27],
        ]);
        assert_eq!(sample_rate, SAMPLE_RATE);
    }

    #[test]
    fn test_wav_bits_per_sample() {
        let bits_per_sample = u16::from_le_bytes([DEFAULT_SOUND_DATA[34], DEFAULT_SOUND_DATA[35]]);
        assert_eq!(bits_per_sample, 16);
    }

    #[test]
    fn test_wav_data_size() {
        let data_size = u32::from_le_bytes([
            DEFAULT_SOUND_DATA[40],
            DEFAULT_SOUND_DATA[41],
            DEFAULT_SOUND_DATA[42],
            DEFAULT_SOUND_DATA[43],
        ]);
        assert_eq!(data_size as usize, NUM_SAMPLES * 2);
    }

    #[test]
    fn test_rodio_can_decode() {
        use rodio::Decoder;
        use std::io::Cursor;

        let cursor = Cursor::new(DEFAULT_SOUND_DATA);
        let result = Decoder::new(cursor);
        assert!(
            result.is_ok(),
            "Rodio should be able to decode the WAV data"
        );
    }
}
