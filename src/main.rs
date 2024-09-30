use advance_runner::run_advance;
use cid::Cid;
use clap::Parser;
use futures::{StreamExt, TryStreamExt};
use hyper::Request;
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient, TryFromUri};
use rs_car_ipfs::single_file::read_single_file_seek;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
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
    dedup_download_directory(&opt.ipfs_url, cid, "machine_source".to_string());
    run_advance(
        String::from("machine_source"),
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

async fn dedup_download_directory(ipfs_url: &str, directory_cid: Cid, out_file_path: String) {
    let ipfs_client = IpfsClient::from_str(ipfs_url).unwrap();
    let res = ipfs_client
        .ls(format!("/ipfs/{}", directory_cid.to_string()).as_str())
        .await
        .unwrap();

    let first_object = res.objects.first().unwrap();

    std::fs::create_dir_all(out_file_path.clone()).unwrap();

    for val in &first_object.links {
        let req = Request::builder()
            .method("POST")
            .uri(format!("{}/api/v0/dag/export?arg={}", ipfs_url, val.hash))
            .body(hyper::Body::empty())
            .unwrap();

        let client = hyper::Client::new();

        match client.request(req).await {
            Ok(res) => {
                let mut f = res
                    .into_body()
                    .map(|result| {
                        result.map_err(|error| {
                            std::io::Error::new(std::io::ErrorKind::Other, "Error!")
                        })
                    })
                    .into_async_read();
                let mut out = async_std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create_new(true)
                    .open(format!("{}/{}", out_file_path, val.name.clone()))
                    .await
                    .unwrap();
                let root_cid = rs_car::Cid::try_from(val.hash.clone()).unwrap();

                read_single_file_seek(&mut f, &mut out, None).await.unwrap();
            }
            Err(er) => {}
        }
    }
}
