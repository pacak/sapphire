//======---------------------------------------------------------------======//
// Copyright (c) 2022 Evan Cox <evanacox00@gmail.com>.                       //
//                                                                           //
// Licensed under the Apache License, Version 2.0 (the "License");           //
// you may not use this file except in compliance with the License.          //
// You may obtain a copy of the License at                                   //
//                                                                           //
//     http://www.apache.org/licenses/LICENSE-2.0                            //
//                                                                           //
// Unless required by applicable law or agreed to in writing, software       //
// distributed under the License is distributed on an "AS IS" BASIS,         //
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.  //
// See the License for the specific language governing permissions and       //
// limitations under the License.                                            //
//======---------------------------------------------------------------======//

#![allow(dead_code)]
#![deny(
    unreachable_pub,
    missing_docs,
    missing_debug_implementations,
    missing_abi,
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links
)]
#![feature(trusted_len)]
#![feature(is_some_and)]
#![feature(fn_traits)]
#![feature(vec_into_raw_parts)]

//! # Sapphire
//!
//! These are the basic APIs for building, manipulating and emitting SIR.

pub mod arena;
pub mod ir;
pub mod utility;