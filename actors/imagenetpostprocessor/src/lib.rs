use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_mlinference::{InferenceOutput, Status};
use wasmcloud_interface_mlimagenet::{Imagenet, ImagenetReceiver, Classification, Matches};

use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use ndarray::{Array, ArrayBase};

mod classes;

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, Imagenet)]
struct ImagenetpostprocessorActor {}

#[async_trait]
impl Imagenet for ImagenetpostprocessorActor {

    async fn postprocess(&self, _ctx: &Context, arg: &InferenceOutput) 
    -> RpcResult<Matches> {
        let tensor = arg.tensor.to_owned();

        if let Status::Error(error) = &arg.result {
            return Err(
                RpcError::InvalidParameter(
                    format!("Invalid input at imagenet postprocessing, due to {:?}", error)
                )
            );
        };

        let raw_result_f32 = bytes_to_f32_vec(tensor.data).unwrap();

        let output_tensor = Array::from_shape_vec((1, 1000, 1, 1), raw_result_f32).unwrap();

        let mut probabilities: Vec<(usize, f32)> = output_tensor
            .softmax(ndarray::Axis(1))
            .into_iter()
            .enumerate()
            .collect::<Vec<_>>();
        
        probabilities.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let labels: Vec<String> = classes::IMAGENT_CLASSES.lines().map(String::from).collect();

        let mut matches: Vec<Classification> = Vec::new();
        
        for i in 0..5 {
            let clfn  = Classification { 
                label: labels[probabilities[i].0].clone(),
                probability: probabilities[i].1,
            };
            
            matches.push(clfn);
        };

        Ok(matches)
    }
}

pub type Result<T> = std::io::Result<T>;

pub fn bytes_to_f32_vec(data: Vec<u8>) -> Result<Vec<f32>> {
    data.chunks(4)
        .into_iter()
        .map(|c| {
            let mut rdr = Cursor::new(c);
            rdr.read_f32::<LittleEndian>()
        })
        .collect()
}

pub trait NdArrayTensor<S, T, D> {
    /// https://en.wikipedia.org/wiki/Softmax_function)
    fn softmax(&self, axis: ndarray::Axis) -> Array<T, D>
    where
        D: ndarray::RemoveAxis,
        S: ndarray::RawData + ndarray::Data + ndarray::RawData<Elem = T>,
        <S as ndarray::RawData>::Elem: std::clone::Clone,
        T: ndarray::NdFloat + std::ops::SubAssign + std::ops::DivAssign;
}

impl<S, T, D> NdArrayTensor<S, T, D> for ArrayBase<S, D>
where
    D: ndarray::RemoveAxis,
    S: ndarray::RawData + ndarray::Data + ndarray::RawData<Elem = T>,
    <S as ndarray::RawData>::Elem: std::clone::Clone,
    T: ndarray::NdFloat + std::ops::SubAssign + std::ops::DivAssign,
{
    fn softmax(&self, axis: ndarray::Axis) -> Array<T, D> {
        let mut new_array: Array<T, D> = self.to_owned();
        new_array.map_inplace(|v| *v = v.exp());
        let sum = new_array.sum_axis(axis).insert_axis(axis);
        new_array /= &sum;

        new_array
    }
}

