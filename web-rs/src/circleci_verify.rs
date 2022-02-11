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

pub fn verify_filter(secret: &str) -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
  let secret = std::sync::Arc::new(secret.to_owned());

  warp::header::<String>(HEADER)
    .and(warp::body::bytes())
    .and_then(move |header: String, body: warp::hyper::body::Bytes| {
      let signature = extract_compatible_signature(&header);
      match signature {
        None => futures::future::err(reject::custom(IncompatibleSignatureVersion)),
        Some(signature) => {
          if verify_signature(&secret, signature, &body) {
            futures::future::ok(())
          } else {
            futures::future::err(reject::custom(InvalidSignature))
          }
        }
      }
    })
    .untuple_one()
}

// struct CircleCiSignature;

// #[rocket::async_trait]
// impl<'r> FromRequest<'r> for CircleCiSignature {
//   type Error = String;
//   async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
//     let body = request;
//     let signature = request.headers().get_one("circle-signature").map(|x| x);
//     match signature {
//       Some(signature) => {
//         let secret = var(CIRCLECI_WEBHOOK_SECRET).unwrap();
//         if verify_signature(&secret, signature, body) {
//           Outcome::Success(CircleCiSignature)
//         } else {
//           Outcome::Failure(())
//         }
//       }
//       None => Outcome::Failure((Status::BadRequest, "Missing circle-signature header")),
//     }
//   }
// }
// import crypto from 'crypto';

// const circleciSignatureVersion = 'v1';

// export function circleciVerify(
//   secret: string,
//   signatureHeader: string,
//   body: string,
// ): boolean {
//   let signatureString = '';

//   const signatures = signatureHeader.split(',');
//   for (const pair of signatures) {
//     const [k, v] = pair.split('=');
//     if (k === circleciSignatureVersion) {
//       signatureString = v;
//       break;
//     }
//   }

//   if (signatureString === '') {
//     throw new Error(
//       'CircleCI signature verification: No compatible signature version found',
//     );
//   }

//   const signature = Buffer.from(signatureString, 'hex');

//   const hmac = crypto.createHmac('sha256', secret);
//   hmac.update(body);
//   const digest = hmac.digest();

//   return crypto.timingSafeEqual(signature, digest);
// }
