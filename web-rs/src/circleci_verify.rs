use hex;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use warp::{reject, Filter};

const SIGNATURE_VERSION: &'static str = "v1";
const HEADER: &'static str = "circleci-signature";

pub fn extract_compatible_signature(header: &str) -> Option<&str> {
    header
        .split(",")
        .find_map(|pair| match pair.split("=").collect::<Vec<&str>>()[..] {
            [v, sig] if v == SIGNATURE_VERSION => Some(sig),
            _ => None,
        })
}

pub fn verify_signature(secret: &str, signature: &str, body: &[u8]) -> bool {
    println!(
        "secret: {}, signature: {}, body: {:?}",
        secret, signature, body
    );

    let mut h = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    h.update(body);
    h.verify(hex::decode(signature).unwrap().as_slice().into())
        .is_ok()
}

#[derive(Debug)]
struct InvalidSignature;
impl reject::Reject for InvalidSignature {}

#[derive(Debug)]
struct IncompatibleSignatureVersion;
impl reject::Reject for IncompatibleSignatureVersion {}

pub fn verify_filter(secret: &str) -> impl Filter<Extract = (), Error = warp::Rejection> + Clone + '_ {
    warp::header::<String>(HEADER)
        .and(warp::body::bytes())
        .and_then(move |header: String, body: warp::hyper::body::Bytes| {
            let signature = extract_compatible_signature(&header);
            match signature {
                None => futures::future::err(reject::custom(IncompatibleSignatureVersion)),
                Some(signature) => {
                    if verify_signature(secret, signature, &body) {
                        futures::future::ok(())
                    } else {
                        futures::future::err(reject::custom(InvalidSignature))
                    }
                }
            }
        })
        .untuple_one()
}
