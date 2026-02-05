use rsa::{
    pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey, EncodeRsaPrivateKey, EncodeRsaPublicKey},
    RsaPrivateKey, RsaPublicKey,
};
use std::fs;
use std::path::Path;

const KEY_SIZE: usize = 2048;

pub struct KeyPair {
    pub private_key: RsaPrivateKey,
    pub public_key: RsaPublicKey,
}

pub fn generate_keys(keys_dir: &Path) -> Result<KeyPair, Box<dyn std::error::Error>> {
    fs::create_dir_all(keys_dir)?;
    let mut rng = rand::thread_rng();
    let private_key = RsaPrivateKey::new(&mut rng, KEY_SIZE)?;
    let public_key = RsaPublicKey::from(&private_key);

    let private_pem = private_key.to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)?;
    let public_pem = public_key.to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)?;

    fs::write(keys_dir.join("private.pem"), private_pem.as_bytes())?;
    fs::write(keys_dir.join("public.pem"), public_pem.as_bytes())?;

    Ok(KeyPair {
        private_key,
        public_key,
    })
}

pub fn load_keys(keys_dir: &Path) -> Result<KeyPair, Box<dyn std::error::Error>> {
    let private_pem = fs::read_to_string(keys_dir.join("private.pem"))?;
    let public_pem = fs::read_to_string(keys_dir.join("public.pem"))?;

    let private_key = RsaPrivateKey::from_pkcs1_pem(&private_pem)?;
    let public_key = RsaPublicKey::from_pkcs1_pem(&public_pem)?;

    Ok(KeyPair {
        private_key,
        public_key,
    })
}

pub fn ensure_keys(keys_dir: &Path) -> Result<KeyPair, Box<dyn std::error::Error>> {
    if keys_dir.join("private.pem").exists() && keys_dir.join("public.pem").exists() {
        load_keys(keys_dir)
    } else {
        generate_keys(keys_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_generate_and_load_keys() {
        let tmp = TempDir::new().unwrap();
        let keys_dir = tmp.path().join("keys");

        let kp = generate_keys(&keys_dir).unwrap();
        assert!(keys_dir.join("private.pem").exists());
        assert!(keys_dir.join("public.pem").exists());

        let kp2 = load_keys(&keys_dir).unwrap();
        assert_eq!(
            kp.public_key
                .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
                .unwrap(),
            kp2.public_key
                .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
                .unwrap()
        );
    }

    #[test]
    fn test_ensure_keys_generates_when_missing() {
        let tmp = TempDir::new().unwrap();
        let keys_dir = tmp.path().join("keys");
        let _kp = ensure_keys(&keys_dir).unwrap();
        assert!(keys_dir.join("private.pem").exists());
    }

    #[test]
    fn test_ensure_keys_loads_existing() {
        let tmp = TempDir::new().unwrap();
        let keys_dir = tmp.path().join("keys");
        let kp1 = generate_keys(&keys_dir).unwrap();
        let kp2 = ensure_keys(&keys_dir).unwrap();
        assert_eq!(
            kp1.public_key
                .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
                .unwrap(),
            kp2.public_key
                .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
                .unwrap()
        );
    }
}
