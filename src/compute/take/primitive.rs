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
    array::{Array, Offset, PrimitiveArray},
    buffer::{Bitmap, Buffer, MutableBitmap, MutableBuffer, NativeType},
    error::Result,
};

use super::maybe_usize;

// take implementation when neither values nor indices contain nulls
fn take_no_validity<T: NativeType, I: Offset>(
    values: &[T],
    indices: &[I],
) -> Result<(Buffer<T>, Option<Bitmap>)> {
    let values = indices
        .iter()
        .map(|index| Result::Ok(values[maybe_usize::<I>(*index)?]));
    // Soundness: `slice.map` is `TrustedLen`.
    let buffer = unsafe { MutableBuffer::try_from_trusted_len_iter(values)? };

    Ok((buffer.into(), None))
}

// take implementation when only values contain nulls
fn take_values_validity<T: NativeType, I: Offset>(
    values: &PrimitiveArray<T>,
    indices: &[I],
) -> Result<(Buffer<T>, Option<Bitmap>)> {
    let mut null = MutableBitmap::with_capacity(indices.len());

    let null_values = values.validity().as_ref().unwrap();

    let values_values = values.values();

    let values = indices.iter().map(|index| {
        let index = maybe_usize::<I>(*index)?;
        if null_values.get_bit(index) {
            null.push(true);
        } else {
            null.push(false);
        }
        Result::Ok(values_values[index])
    });
    // Soundness: `slice.map` is `TrustedLen`.
    let buffer = unsafe { MutableBuffer::try_from_trusted_len_iter(values)? };

    Ok((buffer.into(), null.into()))
}

// take implementation when only indices contain nulls
fn take_indices_validity<T: NativeType, I: Offset>(
    values: &[T],
    indices: &PrimitiveArray<I>,
) -> Result<(Buffer<T>, Option<Bitmap>)> {
    let null_indices = indices.validity().as_ref().unwrap();

    let values = indices.values().iter().map(|index| {
        let index = maybe_usize::<I>(*index)?;
        Result::Ok(match values.get(index) {
            Some(value) => *value,
            None => {
                if null_indices.get_bit(index) {
                    panic!("Out-of-bounds index {}", index)
                } else {
                    T::default()
                }
            }
        })
    });

    // Soundness: `slice.map` is `TrustedLen`.
    let buffer = unsafe { MutableBuffer::try_from_trusted_len_iter(values)? };

    Ok((buffer.into(), indices.validity().clone()))
}

// take implementation when both values and indices contain nulls
fn take_values_indices_validity<T: NativeType, I: Offset>(
    values: &PrimitiveArray<T>,
    indices: &PrimitiveArray<I>,
) -> Result<(Buffer<T>, Option<Bitmap>)> {
    let mut bitmap = MutableBitmap::with_capacity(indices.len());

    let values_validity = values.validity().as_ref().unwrap();

    let values_values = values.values();
    let values = indices.iter().map(|index| match index {
        Some(index) => {
            let index = maybe_usize::<I>(index)?;
            bitmap.push(values_validity.get_bit(index));
            Result::Ok(values_values[index])
        }
        None => {
            bitmap.push(false);
            Ok(T::default())
        }
    });
    // Soundness: `slice.map` is `TrustedLen`.
    let buffer = unsafe { MutableBuffer::try_from_trusted_len_iter(values)? };
    Ok((buffer.into(), bitmap.into()))
}

/// `take` implementation for primitive arrays
pub fn take<T: NativeType, I: Offset>(
    values: &PrimitiveArray<T>,
    indices: &PrimitiveArray<I>,
) -> Result<PrimitiveArray<T>> {
    let indices_has_validity = indices.null_count() > 0;
    let values_has_validity = values.null_count() > 0;
    // note: this function should only panic when "an index is not null and out of bounds".
    // if the index is null, its value is undefined and therefore we should not read from it.

    let (buffer, nulls) = match (values_has_validity, indices_has_validity) {
        (false, false) => {
            // * no nulls
            // * all `indices.values()` are valid
            take_no_validity::<T, I>(values.values(), indices.values())?
        }
        (true, false) => {
            // * nulls come from `values` alone
            // * all `indices.values()` are valid
            take_values_validity::<T, I>(values, indices.values())?
        }
        (false, true) => {
            // in this branch it is unsound to read and use `index.values()`,
            // as doing so is UB when they come from a null slot.
            take_indices_validity::<T, I>(values.values(), indices)?
        }
        (true, true) => {
            // in this branch it is unsound to read and use `index.values()`,
            // as doing so is UB when they come from a null slot.
            take_values_indices_validity::<T, I>(values, indices)?
        }
    };

    Ok(PrimitiveArray::<T>::from_data(
        values.data_type().clone(),
        buffer,
        nulls,
    ))
}