use std::io::Cursor;
use std::cmp::Ordering;
use wasmbus_rpc::actor::prelude::*;
use byteorder::{LittleEndian, ReadBytesExt};
use wasmcloud_interface_mlinference::{InferenceOutput, Status};
use wasmcloud_interface_mlimagenet::{Classification, Imagenet, ImagenetReceiver, Matches};

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, Imagenet)]
struct MNistpostprocessorActor {}

pub type Result<T> = std::io::Result<T>;

async fn max_by_index(v: &Vec<f32>) -> Option<usize> {
    let index_of_max: Option<usize> = v
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
        .map(|(index, _)| index);
    
    return index_of_max;
}

async fn bytes_to_f32_vec(data: Vec<u8>) -> Result<Vec<f32>> {
    data.chunks(4)
        .into_iter()
        .map(|c| {
            let mut rdr = Cursor::new(c);
            rdr.read_f32::<LittleEndian>()
        })
        .collect()
}

#[async_trait]
impl Imagenet for MNistpostprocessorActor {
    async fn postprocess(&self, _ctx: &Context, arg: &InferenceOutput) -> RpcResult<Matches> {
        let tensor: Vec<f32> = bytes_to_f32_vec(arg.tensor.to_owned().data).await?;

        if let Status::Error(error) = &arg.result {
            return Err(RpcError::InvalidParameter(format!(
                "Invalid input at postprocessing of mnist data due to {:?}",
                error
            )));
        };

        //let (guess, confidence) = tensor.iter().enumerate().max_by(|x, y| x.1.cmp(y.1)).unwrap();
        //let result = tensor.into_iter().enumerate();
        let index_of_max = max_by_index(&tensor).await.unwrap();
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
