use byteorder::{LittleEndian, ReadBytesExt};
use std::cmp::Ordering;
use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_mlimagenet::{Classification, Imagenet, ImagenetReceiver, Matches};
use wasmcloud_interface_mlinference::{InferenceOutput, Status};

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, Imagenet)]
struct MNistpostprocessorActor {}

pub type Result<T> = std::io::Result<T>;

fn max_by_index(v: &[f32]) -> Option<usize> {
    let index_of_max: Option<usize> = v
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
        .map(|(index, _)| index);

    return index_of_max;
}

fn bytes_to_f32_vec<T: AsRef<[u8]>>(data: T) -> RpcResult<Vec<f32>> {
    let mut rdr = std::io::Cursor::new(data.as_ref());
    let mut buf = Vec::with_capacity(data.as_ref().len() / 4);
    while let Ok(v) = rdr.read_f32::<LittleEndian>() {
        buf.push(v);
    }
    if buf.len() * 4 < data.as_ref().len() {
        Err(RpcError::Other(format!(
            "data conversion error at offset {}",
            buf.len() >> 2
        )))
    } else {
        Ok(buf)
    }
}

#[async_trait]
impl Imagenet for MNistpostprocessorActor {
    async fn postprocess(&self, _ctx: &Context, arg: &InferenceOutput) -> RpcResult<Matches> {
        let tensor: Vec<f32> = bytes_to_f32_vec(&arg.tensor.data)?;

        if let Status::Error(error) = &arg.result {
            return Err(RpcError::InvalidParameter(format!(
                "Invalid input at postprocessing of mnist data due to {:?}",
                error
            )));
        };

        //let (guess, confidence) = tensor.iter().enumerate().max_by(|x, y| x.1.cmp(y.1)).unwrap();
        //let result = tensor.into_iter().enumerate();
        let index_of_max = max_by_index(&tensor).unwrap();
        let confidence = tensor.get(index_of_max).or(Some(&-1.0)).unwrap();

        let mut matches: Vec<Classification> = Vec::new();
        let clfn = Classification {
            label: index_of_max.to_string(),
            probability: confidence.to_owned(),
        };

        matches.push(clfn);

        // for i in 0..5 {
        //     let clfn = Classification {
        //         label: labels[probabilities[i].0].clone(),
        //         probability: probabilities[i].1,
        //     };
        //     matches.push(clfn);
        // }

        Ok(matches)
    }
}
