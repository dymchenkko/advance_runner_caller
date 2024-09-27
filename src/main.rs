use advance_runner::run_advance;
use cid::Cid;
use clap::Parser;
use futures_util::TryStreamExt;
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient, TryFromUri};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Error, ErrorKind, Write};

#[derive(Parser, Clone, Debug)]
pub struct Options {
    #[clap(long, env = "IPFS_URL")]
    pub ipfs_url: String,

    #[clap(long, env = "PAYLOAD", num_args = 1.., value_delimiter = ' ')]
    pub payload: Vec<u8>,

    #[clap(long, env = "CID")]
    pub cid: String,
}

#[async_std::main]
async fn main() {
    let opt = Options::parse();
    let cid = Cid::try_from(opt.cid).unwrap();
    let ipfs_client = IpfsClient::from_str(&opt.ipfs_url).unwrap();
    match ipfs_client
        .get(&format!("{}/gov/app/info.json", cid.to_string()))
        .map_ok(|chunk| chunk.to_vec())
        .try_concat()
        .await
    {
        Ok(res) => {
            let mut file = match File::create("info.json") {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("error creating file: {}", e);
                    return;
                }
            };

            if let Err(e) = file.write_all(&res) {
                eprintln!("error writing to file: {}", e);
            }
        }
        Err(e) => eprintln!("error getting file: {}", e),
    }
    run_advance(
        String::from("info.json"),
        "lambda_state_previous path",
        "lambda_state_next path",
        opt.payload,
        HashMap::new(),
        Box::new(report_callback),
        Box::new(output_callback),
        HashMap::new(),
    )
    .unwrap();
}

fn report_callback(reason: u16, payload: &[u8]) -> Result<(u16, Vec<u8>), Error> {
    return Err(Error::from(ErrorKind::UnexpectedEof));
}

fn output_callback(reason: u16, payload: &[u8]) -> Result<(u16, Vec<u8>), Error> {
    return Err(Error::from(ErrorKind::UnexpectedEof));
}
