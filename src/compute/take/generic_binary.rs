// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.
use crate::{
    array::{Array, GenericBinaryArray, Offset, PrimitiveArray},
    bitmap::{Bitmap, MutableBitmap},
    buffer::{Buffer, MutableBuffer},
};

use super::Index;

pub fn take_values<O: Offset>(length: O, starts: &[O], offsets: &[O], values: &[u8]) -> Buffer<u8> {
    let new_len = length.to_usize();
    let mut buffer = MutableBuffer::with_capacity(new_len);
    starts
        .iter()
        .zip(offsets.windows(2))
        .for_each(|(start_, window)| {
            let start = start_.to_usize();
            let end = (*start_ + (window[1] - window[0])).to_usize();
            buffer.extend_from_slice(&values[start..end]);
        });
    buffer.into()
}

// take implementation when neither values nor indices contain nulls
pub fn take_no_validity<O: Offset, I: Index>(
    offsets: &[O],
    values: &[u8],
    indices: &[I],
) -> (Buffer<O>, Buffer<u8>, Option<Bitmap>) {
    let mut length = O::default();
    let mut buffer = MutableBuffer::<u8>::new();
    let offsets = indices.iter().map(|index| {
        let index = index.to_usize();
        let start = offsets[index];
        let length_h = offsets[index + 1] - start;
        length += length_h;

        let _start = start.to_usize();
        let end = (start + length_h).to_usize();
        buffer.extend_from_slice(&values[_start..end]);
        length
    });
    let offsets = std::iter::once(O::default()).chain(offsets);
    let offsets = Buffer::from_trusted_len_iter(offsets);

    (offsets, buffer.into(), None)
}

// take implementation when only values contain nulls
pub fn take_values_validity<O: Offset, I: Index, A: GenericBinaryArray<O>>(
    values: &A,
    indices: &[I],
) -> (Buffer<O>, Buffer<u8>, Option<Bitmap>) {
    let mut length = O::default();
    let mut validity = MutableBitmap::with_capacity(indices.len());

    let null_values = values.validity().as_ref().unwrap();
    let offsets = values.offsets();
    let values_values = values.values();

    let mut starts = MutableBuffer::<O>::with_capacity(indices.len());
    let offsets = indices.iter().map(|index| {
        let index = index.to_usize();
        if null_values.get_bit(index) {
            validity.push(true);
            let start = offsets[index];
            length += offsets[index + 1] - start;
            starts.push(start);
        } else {
            validity.push(false);
            starts.push(O::default());
        }
        length
    });
    let offsets = std::iter::once(O::default()).chain(offsets);
    let offsets = Buffer::from_trusted_len_iter(offsets);

    let buffer = take_values(length, starts.as_slice(), offsets.as_slice(), values_values);

    (offsets, buffer, validity.into())
}

// take implementation when only indices contain nulls
pub fn take_indices_validity<O: Offset, I: Index>(
    offsets: &[O],
    values: &[u8],
    indices: &PrimitiveArray<I>,
) -> (Buffer<O>, Buffer<u8>, Option<Bitmap>) {
    let mut length = O::default();

    let mut starts = MutableBuffer::<O>::with_capacity(indices.len());
    let offsets = indices.values().iter().map(|index| {
        let index = index.to_usize();
        match offsets.get(index + 1) {
            Some(&next) => {
                let start = offsets[index];
                length += next - start;
                starts.push(start);
            }
            None => starts.push(O::default()),
        };
        length
    });
    let offsets = std::iter::once(O::default()).chain(offsets);
    let offsets = Buffer::from_trusted_len_iter(offsets);
    let starts: Buffer<O> = starts.into();

    let buffer = take_values(length, starts.as_slice(), offsets.as_slice(), values);

    (offsets, buffer, indices.validity().clone())
}

// take implementation when both indices and values contain nulls
pub fn take_values_indices_validity<O: Offset, I: Index, A: GenericBinaryArray<O>>(
    values: &A,
    indices: &PrimitiveArray<I>,
) -> (Buffer<O>, Buffer<u8>, Option<Bitmap>) {
    let mut length = O::default();
    let mut validity = MutableBitmap::with_capacity(indices.len());

    let values_validity = values.validity().as_ref().unwrap();
    let offsets = values.offsets();
    let values_values = values.values();

    let mut starts = MutableBuffer::<O>::with_capacity(indices.len());
    let offsets = indices.iter().map(|index| {
        match index {
            Some(index) => {
                let index = index.to_usize();
                if values_validity.get_bit(index) {
                    validity.push(true);
                    length += offsets[index + 1] - offsets[index];
                    starts.push(offsets[index]);
                } else {
                    validity.push(false);
                    starts.push(O::default());
                }
            }
            None => {
                validity.push(false);
                starts.push(O::default());
            }
        };
        length
    });
    let offsets = std::iter::once(O::default()).chain(offsets);
    let offsets = Buffer::from_trusted_len_iter(offsets);
    let starts: Buffer<O> = starts.into();

    let buffer = take_values(length, starts.as_slice(), offsets.as_slice(), values_values);

    (offsets, buffer, validity.into())
}
