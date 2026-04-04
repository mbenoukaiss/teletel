use bevy::prelude::*;

#[derive(Resource)]
pub(super) struct BeepSound(pub Handle<AudioSource>);

const SAMPLE_RATE: u32 = 44100;
const FREQUENCY: u32 = 440;
const DURATION_MS: u32 = 200;
const NUM_SAMPLES: usize = (SAMPLE_RATE * DURATION_MS / 1000) as usize;
const WAV_SIZE: usize = 44 + NUM_SAMPLES * 2;

const fn build_wav() -> [u8; WAV_SIZE] {
    let mut wav = [0u8; WAV_SIZE];
    let data_size = (NUM_SAMPLES * 2) as u32;
    let file_size = 36 + data_size;

    // riff header
    wav[0] = b'R'; wav[1] = b'I'; wav[2] = b'F'; wav[3] = b'F';
    let fs = file_size.to_le_bytes();
    wav[4] = fs[0]; wav[5] = fs[1]; wav[6] = fs[2]; wav[7] = fs[3];
    wav[8] = b'W'; wav[9] = b'A'; wav[10] = b'V'; wav[11] = b'E';

    // fmt chunk
    wav[12] = b'f'; wav[13] = b'm'; wav[14] = b't'; wav[15] = b' ';
    wav[16] = 16;
    wav[20] = 1; // pcm
    wav[22] = 1; // mono
    let sr = SAMPLE_RATE.to_le_bytes();
    wav[24] = sr[0]; wav[25] = sr[1]; wav[26] = sr[2]; wav[27] = sr[3];
    let br = (SAMPLE_RATE * 2).to_le_bytes();
    wav[28] = br[0]; wav[29] = br[1]; wav[30] = br[2]; wav[31] = br[3];
    wav[32] = 2; // block align
    wav[34] = 16; // bits per sample

    // data chunk
    wav[36] = b'd'; wav[37] = b'a'; wav[38] = b't'; wav[39] = b'a';
    let ds = data_size.to_le_bytes();
    wav[40] = ds[0]; wav[41] = ds[1]; wav[42] = ds[2]; wav[43] = ds[3];

    let half_period = SAMPLE_RATE / FREQUENCY / 2;
    let amplitude: i16 = 9830; // ~30% of i16::MAX
    let mut i = 0;
    while i < NUM_SAMPLES {
        let phase = (i as u32 % (half_period * 2)) < half_period;
        let sample = if phase { amplitude } else { -amplitude };
        let s = sample.to_le_bytes();
        wav[44 + i * 2] = s[0];
        wav[44 + i * 2 + 1] = s[1];
        i += 1;
    }

    wav
}

static BEEP_WAV: [u8; WAV_SIZE] = build_wav();

pub(super) fn setup_beep_sound(
    mut commands: Commands,
    mut audio_assets: ResMut<Assets<AudioSource>>,
) {
    let source = AudioSource { bytes: BEEP_WAV[..].into() };
    let handle = audio_assets.add(source);
    commands.insert_resource(BeepSound(handle));
}
