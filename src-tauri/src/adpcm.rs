pub const SAMPLES_PER_PACKET: usize = 256;
pub const DATA_LEN: usize = SAMPLES_PER_PACKET / 2;
pub const PACKET_LEN: usize = 1 + 5 + DATA_LEN;
pub const AUDIO_PACKET: u8 = 0x10;

const INDEX_TABLE: [i32; 8] = [-1, -1, -1, -1, 2, 4, 6, 8];

const STEP_TABLE: [i32; 89] = [
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60, 66,
    73, 80, 88, 97, 107, 118, 130, 143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371, 408, 449,
    494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066, 2272,
    2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358, 5894, 6484, 7132, 7845, 8630, 9493,
    10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623, 27086, 29794, 32767,
];

#[derive(Debug, Clone, Copy, Default)]
struct State {
    predictor: i32,
    index: i32,
}

impl State {
    fn apply(&mut self, code: u8) {
        let step = STEP_TABLE[self.index as usize];
        let mut delta = step >> 3;
        if code & 4 != 0 {
            delta += step;
        }
        if code & 2 != 0 {
            delta += step >> 1;
        }
        if code & 1 != 0 {
            delta += step >> 2;
        }
        if code & 8 != 0 {
            self.predictor -= delta;
        } else {
            self.predictor += delta;
        }
        self.predictor = self.predictor.clamp(i16::MIN as i32, i16::MAX as i32);
        self.index = (self.index + INDEX_TABLE[(code & 7) as usize]).clamp(0, 88);
    }

    fn encode(&mut self, sample: i16) -> u8 {
        let step = STEP_TABLE[self.index as usize];
        let mut diff = sample as i32 - self.predictor;
        let mut code = 0u8;
        if diff < 0 {
            code = 8;
            diff = -diff;
        }
        let mut threshold = step;
        if diff >= threshold {
            code |= 4;
            diff -= threshold;
        }
        threshold >>= 1;
        if diff >= threshold {
            code |= 2;
            diff -= threshold;
        }
        threshold >>= 1;
        if diff >= threshold {
            code |= 1;
        }
        self.apply(code);
        code
    }
}

#[derive(Default)]
pub struct Encoder {
    state: State,
    seq: u16,
}

impl Encoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn encode(&mut self, samples: &[i16]) -> Vec<u8> {
        let mut packet = Vec::with_capacity(PACKET_LEN);
        packet.push(AUDIO_PACKET);
        packet.extend_from_slice(&self.seq.to_le_bytes());
        packet.extend_from_slice(&(self.state.predictor as i16).to_le_bytes());
        packet.push(self.state.index as u8);

        for pair in 0..DATA_LEN {
            let low = self.state.encode(*samples.get(pair * 2).unwrap_or(&0));
            let high = self.state.encode(*samples.get(pair * 2 + 1).unwrap_or(&0));
            packet.push(low | (high << 4));
        }

        self.seq = self.seq.wrapping_add(1);
        packet
    }
}

pub fn decode(packet: &[u8]) -> Option<(u16, Vec<i16>)> {
    if packet.len() != PACKET_LEN || packet[0] != AUDIO_PACKET {
        return None;
    }
    let seq = u16::from_le_bytes([packet[1], packet[2]]);
    let index = packet[5] as i32;
    if index > 88 {
        return None;
    }
    let mut state = State {
        predictor: i16::from_le_bytes([packet[3], packet[4]]) as i32,
        index,
    };

    let mut samples = Vec::with_capacity(SAMPLES_PER_PACKET);
    for byte in &packet[6..] {
        state.apply(byte & 0x0F);
        samples.push(state.predictor as i16);
        state.apply(byte >> 4);
        samples.push(state.predictor as i16);
    }
    Some((seq, samples))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(length: usize) -> Vec<i16> {
        (0..length)
            .map(|n| ((n as f32 * 0.1).sin() * 12000.0) as i16)
            .collect()
    }

    #[test]
    fn round_trip_stays_close() {
        let input = tone(SAMPLES_PER_PACKET * 4);
        let mut encoder = Encoder::new();
        let mut output = Vec::new();
        for block in input.chunks(SAMPLES_PER_PACKET) {
            let packet = encoder.encode(block);
            assert_eq!(packet.len(), PACKET_LEN);
            output.extend(decode(&packet).unwrap().1);
        }
        let error = input
            .iter()
            .zip(&output)
            .skip(64)
            .map(|(a, b)| (*a as i32 - *b as i32).abs())
            .max()
            .unwrap();
        assert!(error < 1200, "erro máximo {error}");
    }

    #[test]
    fn lost_packet_does_not_corrupt_next() {
        let input = tone(SAMPLES_PER_PACKET * 3);
        let mut encoder = Encoder::new();
        let packets: Vec<_> = input
            .chunks(SAMPLES_PER_PACKET)
            .map(|block| encoder.encode(block))
            .collect();
        let (seq, samples) = decode(&packets[2]).unwrap();
        assert_eq!(seq, 2);
        let error = input[SAMPLES_PER_PACKET * 2..]
            .iter()
            .zip(&samples)
            .map(|(a, b)| (*a as i32 - *b as i32).abs())
            .max()
            .unwrap();
        assert!(error < 1200, "erro máximo {error}");
    }

    #[test]
    fn rejects_malformed_packets() {
        assert!(decode(&[AUDIO_PACKET, 0, 0]).is_none());
        assert!(decode(&vec![0u8; PACKET_LEN]).is_none());
    }
}
