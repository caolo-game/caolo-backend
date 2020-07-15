use rand::RngCore;

pub fn generate_refresh_token(len: usize) -> String {
    let mut rng = rand::thread_rng();
    let mut randid = vec![0; len];
    rng.fill_bytes(&mut randid);
    let randid = base64::encode(&randid);
    randid
}
