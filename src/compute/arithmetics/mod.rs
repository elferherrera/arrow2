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

//! Defines basic arithmetic kernels for `PrimitiveArrays`.

use std::ops::{Add, Div, Mul, Neg, Sub};

use num::{traits::Pow, Zero};

use crate::array::*;
use crate::buffer::Buffer;
use crate::datatypes::DataType;
use crate::error::{ArrowError, Result};
use crate::types::NativeType;

use super::arity::{binary, unary};
use super::utils::combine_validities;

pub fn arithmetic(lhs: &dyn Array, op: Operator, rhs: &dyn Array) -> Result<Box<dyn Array>> {
    let data_type = lhs.data_type();
    if data_type != rhs.data_type() {
        return Err(ArrowError::NotYetImplemented(
            "Arithmetic is currently only supported for arrays of the same logical type"
                .to_string(),
        ));
    }
    match data_type {
        DataType::Int8 => {
            let lhs = lhs.as_any().downcast_ref::<Int8Array>().unwrap();
            let rhs = rhs.as_any().downcast_ref::<Int8Array>().unwrap();
            arithmetic_primitive(lhs, op, rhs)
                .map(Box::new)
                .map(|x| x as Box<dyn Array>)
        }
        DataType::Int16 => {
            let lhs = lhs.as_any().downcast_ref::<Int16Array>().unwrap();
            let rhs = rhs.as_any().downcast_ref::<Int16Array>().unwrap();
            arithmetic_primitive(lhs, op, rhs)
                .map(Box::new)
                .map(|x| x as Box<dyn Array>)
        }
        DataType::Int32 => {
            let lhs = lhs.as_any().downcast_ref::<Int32Array>().unwrap();
            let rhs = rhs.as_any().downcast_ref::<Int32Array>().unwrap();
            arithmetic_primitive(lhs, op, rhs)
                .map(Box::new)
                .map(|x| x as Box<dyn Array>)
        }
        DataType::Int64 | DataType::Duration(_) => {
            let lhs = lhs.as_any().downcast_ref::<Int64Array>().unwrap();
            let rhs = rhs.as_any().downcast_ref::<Int64Array>().unwrap();
            arithmetic_primitive(lhs, op, rhs)
                .map(Box::new)
                .map(|x| x as Box<dyn Array>)
        }
        DataType::UInt8 => {
            let lhs = lhs.as_any().downcast_ref::<UInt8Array>().unwrap();
            let rhs = rhs.as_any().downcast_ref::<UInt8Array>().unwrap();
            arithmetic_primitive(lhs, op, rhs)
                .map(Box::new)
                .map(|x| x as Box<dyn Array>)
        }
        DataType::UInt16 => {
            let lhs = lhs.as_any().downcast_ref::<UInt16Array>().unwrap();
            let rhs = rhs.as_any().downcast_ref::<UInt16Array>().unwrap();
            arithmetic_primitive(lhs, op, rhs)
                .map(Box::new)
                .map(|x| x as Box<dyn Array>)
        }
        DataType::UInt32 => {
            let lhs = lhs.as_any().downcast_ref::<UInt32Array>().unwrap();
            let rhs = rhs.as_any().downcast_ref::<UInt32Array>().unwrap();
            arithmetic_primitive(lhs, op, rhs)
                .map(Box::new)
                .map(|x| x as Box<dyn Array>)
        }
        DataType::UInt64 => {
            let lhs = lhs.as_any().downcast_ref::<UInt64Array>().unwrap();
            let rhs = rhs.as_any().downcast_ref::<UInt64Array>().unwrap();
            arithmetic_primitive(lhs, op, rhs)
                .map(Box::new)
                .map(|x| x as Box<dyn Array>)
        }
        DataType::Float16 => unreachable!(),
        DataType::Float32 => {
            let lhs = lhs.as_any().downcast_ref::<Float32Array>().unwrap();
            let rhs = rhs.as_any().downcast_ref::<Float32Array>().unwrap();
            arithmetic_primitive(lhs, op, rhs)
                .map(Box::new)
                .map(|x| x as Box<dyn Array>)
        }
        DataType::Float64 => {
            let lhs = lhs.as_any().downcast_ref::<Float64Array>().unwrap();
            let rhs = rhs.as_any().downcast_ref::<Float64Array>().unwrap();
            arithmetic_primitive(lhs, op, rhs)
                .map(Box::new)
                .map(|x| x as Box<dyn Array>)
        }
        DataType::Decimal(_, _) => {
            let lhs = lhs.as_any().downcast_ref::<Int128Array>().unwrap();
            let rhs = rhs.as_any().downcast_ref::<Int128Array>().unwrap();
            arithmetic_primitive(lhs, op, rhs)
                .map(Box::new)
                .map(|x| x as Box<dyn Array>)
        }
        _ => Err(ArrowError::NotYetImplemented(format!(
            "Arithmetics between {:?} is not supported",
            data_type
        ))),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[inline]
fn arithmetic_primitive<T>(
    lhs: &PrimitiveArray<T>,
    op: Operator,
    rhs: &PrimitiveArray<T>,
) -> Result<PrimitiveArray<T>>
where
    T: NativeType + Div<Output = T> + Zero + Add<Output = T> + Sub<Output = T> + Mul<Output = T>,
{
    match op {
        Operator::Add => add(lhs, rhs),
        Operator::Subtract => subtract(lhs, rhs),
        Operator::Multiply => multiply(lhs, rhs),
        Operator::Divide => divide(lhs, rhs),
    }
}

#[inline]
pub fn arithmetic_primitive_scalar<T>(
    lhs: &PrimitiveArray<T>,
    op: Operator,
    rhs: &T,
) -> Result<PrimitiveArray<T>>
where
    T: NativeType + Div<Output = T> + Zero + Add<Output = T> + Sub<Output = T> + Mul<Output = T>,
{
    match op {
        Operator::Add => Ok(add_scalar(lhs, rhs)),
        Operator::Subtract => Ok(subtract_scalar(lhs, rhs)),
        Operator::Multiply => Ok(multiply_scalar(lhs, rhs)),
        Operator::Divide => divide_scalar(lhs, rhs),
    }
}

/// Divide two arrays.
///
/// # Errors
///
/// This function errors iff:
/// * the arrays have different lengths
/// * a division by zero is found
fn divide<T>(lhs: &PrimitiveArray<T>, rhs: &PrimitiveArray<T>) -> Result<PrimitiveArray<T>>
where
    T: NativeType,
    T: Div<Output = T> + Zero,
{
    if lhs.len() != rhs.len() {
        return Err(ArrowError::InvalidArgumentError(
            "Cannot perform math operation on arrays of different length".to_string(),
        ));
    }

    let validity = combine_validities(lhs.validity(), rhs.validity());

    let values = if let Some(b) = &validity {
        // there are nulls. Division by zero on them should be ignored
        let values =
            b.iter()
                .zip(lhs.values().iter().zip(rhs.values()))
                .map(|(is_valid, (lhs, rhs))| {
                    if is_valid {
                        if rhs.is_zero() {
                            Err(ArrowError::InvalidArgumentError(
                                "There is a zero in the division, causing a division by zero"
                                    .to_string(),
                            ))
                        } else {
                            Ok(*lhs / *rhs)
                        }
                    } else {
                        Ok(T::default())
                    }
                });
        unsafe { Buffer::try_from_trusted_len_iter(values) }
    } else {
        // no value is null
        let values = lhs.values().iter().zip(rhs.values()).map(|(lhs, rhs)| {
            if rhs.is_zero() {
                Err(ArrowError::InvalidArgumentError(
                    "There is a zero in the division, causing a division by zero".to_string(),
                ))
            } else {
                Ok(*lhs / *rhs)
            }
        });
        unsafe { Buffer::try_from_trusted_len_iter(values) }
    }?;

    Ok(PrimitiveArray::<T>::from_data(
        lhs.data_type().clone(),
        values,
        validity,
    ))
}

/// Divide an array by a constant
pub fn divide_scalar<T>(array: &PrimitiveArray<T>, divisor: &T) -> Result<PrimitiveArray<T>>
where
    T: NativeType,
    T: Div<Output = T> + Zero,
{
    if divisor.is_zero() {
        return Err(ArrowError::InvalidArgumentError(
            "The divisor cannot be zero".to_string(),
        ));
    }
    let divisor = *divisor;
    Ok(unary(array, |x| x / divisor, array.data_type()))
}

#[inline]
pub fn add<T>(lhs: &PrimitiveArray<T>, rhs: &PrimitiveArray<T>) -> Result<PrimitiveArray<T>>
where
    T: NativeType + Add<Output = T>,
{
    binary(lhs, rhs, |a, b| a + b)
}

#[inline]
fn add_scalar<T>(lhs: &PrimitiveArray<T>, rhs: &T) -> PrimitiveArray<T>
where
    T: NativeType + Add<Output = T>,
{
    let rhs = *rhs;
    unary(lhs, |a| a + rhs, lhs.data_type())
}

#[inline]
fn subtract<T>(lhs: &PrimitiveArray<T>, rhs: &PrimitiveArray<T>) -> Result<PrimitiveArray<T>>
where
    T: NativeType + Sub<Output = T>,
{
    binary(lhs, rhs, |a, b| a - b)
}

#[inline]
fn subtract_scalar<T>(lhs: &PrimitiveArray<T>, rhs: &T) -> PrimitiveArray<T>
where
    T: NativeType + Sub<Output = T>,
{
    let rhs = *rhs;
    unary(lhs, |a| a - rhs, lhs.data_type())
}

#[inline]
pub fn negate<T>(array: &PrimitiveArray<T>) -> PrimitiveArray<T>
where
    T: NativeType + Neg<Output = T>,
{
    unary(array, |a| -a, array.data_type())
}

#[inline]
pub fn powf_scalar<T>(array: &PrimitiveArray<T>, exponent: T) -> PrimitiveArray<T>
where
    T: NativeType + Pow<T, Output = T>,
{
    unary(array, |x| x.pow(exponent), array.data_type())
}

#[inline]
fn multiply<T>(lhs: &PrimitiveArray<T>, rhs: &PrimitiveArray<T>) -> Result<PrimitiveArray<T>>
where
    T: NativeType + Mul<Output = T>,
{
    binary(lhs, rhs, |lhs, rhs| lhs * rhs)
}

#[inline]
pub fn multiply_scalar<T>(lhs: &PrimitiveArray<T>, rhs: &T) -> PrimitiveArray<T>
where
    T: NativeType + Mul<Output = T>,
{
    let rhs = *rhs;
    unary(lhs, |lhs| lhs * rhs, lhs.data_type())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datatypes::DataType;

    #[test]
    fn test_add() {
        let a = Primitive::from(&vec![None, Some(6), None, Some(6)]).to(DataType::Int32);
        let b = Primitive::from(&vec![Some(5), None, None, Some(6)]).to(DataType::Int32);
        let result = add(&a, &b).unwrap();
        let expected = Primitive::from(&vec![None, None, None, Some(12)]).to(DataType::Int32);
        assert_eq!(result, expected)
    }

    #[test]
    fn test_add_mismatched_length() {
        let a = Primitive::from_slice(vec![5, 6]).to(DataType::Int32);
        let b = Primitive::from_slice(vec![5]).to(DataType::Int32);
        add(&a, &b)
            .err()
            .expect("should have failed due to different lengths");
    }

    #[test]
    fn test_subtract() {
        let a = Primitive::from(&vec![None, Some(6), None, Some(6)]).to(DataType::Int32);
        let b = Primitive::from(&vec![Some(5), None, None, Some(6)]).to(DataType::Int32);
        let result = subtract(&a, &b).unwrap();
        let expected = Primitive::from(&vec![None, None, None, Some(0)]).to(DataType::Int32);
        assert_eq!(result, expected)
    }

    #[test]
    fn test_multiply() {
        let a = Primitive::from(&vec![None, Some(6), None, Some(6)]).to(DataType::Int32);
        let b = Primitive::from(&vec![Some(5), None, None, Some(6)]).to(DataType::Int32);
        let result = multiply(&a, &b).unwrap();
        let expected = Primitive::from(&vec![None, None, None, Some(36)]).to(DataType::Int32);
        assert_eq!(result, expected)
    }

    #[test]
    fn test_divide() {
        let a = Primitive::from(&vec![None, Some(6), None, Some(6)]).to(DataType::Int32);
        let b = Primitive::from(&vec![Some(5), None, None, Some(6)]).to(DataType::Int32);
        let result = divide(&a, &b).unwrap();
        let expected = Primitive::from(&vec![None, None, None, Some(1)]).to(DataType::Int32);
        assert_eq!(result, expected)
    }

    #[test]
    fn test_divide_scalar() {
        let a = Primitive::from(&vec![None, Some(6)]).to(DataType::Int32);
        let b = 3i32;
        let result = divide_scalar(&a, &b).unwrap();
        let expected = Primitive::from(&vec![None, Some(2)]).to(DataType::Int32);
        assert_eq!(result, expected)
    }

    #[test]
    fn test_divide_by_zero() {
        let a = Primitive::from(&vec![Some(6)]).to(DataType::Int32);
        let b = Primitive::from(&vec![Some(0)]).to(DataType::Int32);
        assert_eq!(divide(&a, &b).is_err(), true);
    }

    #[test]
    fn test_divide_by_zero_on_null() {
        let a = Primitive::from(&vec![None]).to(DataType::Int32);
        let b = Primitive::from(&vec![Some(0)]).to(DataType::Int32);
        assert_eq!(divide(&a, &b).is_err(), false);
    }

    #[test]
    fn test_raise_power_scalar() {
        let a = Primitive::from(&vec![Some(2f32), None]).to(DataType::Float32);
        let actual = powf_scalar(&a, 2.0);
        let expected = Primitive::from(&vec![Some(4f32), None]).to(DataType::Float32);
        assert_eq!(expected, actual);
    }
}
