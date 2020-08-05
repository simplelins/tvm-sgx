/*
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

extern crate tvm_runtime;
use std::{
    convert::TryFrom as _,
    io::{Read as _, Write as _},
};
#[cfg(target_env = "sgx")]
extern "C" {
    fn model1___tvm_module_startup();
    fn model2___tvm_module_startup();
}
fn main() {
    let syslib = tvm_runtime::SystemLibModule::default();

    let graph_json = include_str!(concat!(env!("OUT_DIR"), "/graph.json"));
    let params_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/params.bin"));    
    let params = tvm_runtime::load_param_dict(params_bytes).unwrap();
    let graph = tvm_runtime::Graph::try_from(graph_json).unwrap();

    let graph_json1 = include_str!(concat!(env!("OUT_DIR"), "/graph1.json"));
    let params_bytes1 = include_bytes!(concat!(env!("OUT_DIR"), "/params1.bin"));
    let params1 = tvm_runtime::load_param_dict(params_bytes1).unwrap();
    let graph1 = tvm_runtime::Graph::try_from(graph_json1).unwrap();

    let mut exec = tvm_runtime::GraphExecutor::new(graph, &syslib).unwrap();
    exec.load_params(params);

    // let startup1 = syslib
    //     .get_function("__tvm_module_startup1")
    //     .expect("main function not found");
    unsafe {
        model1___tvm_module_startup();
    }
    let mut exec1 = tvm_runtime::GraphExecutor::new(graph1, &syslib).unwrap();
    exec1.load_params(params1);

    unsafe {
        model2___tvm_module_startup();
    }
    let graph_json2 = include_str!(concat!(env!("OUT_DIR"), "/graph2.json"));
    let params_bytes2 = include_bytes!(concat!(env!("OUT_DIR"), "/params2.bin"));
    let params2 = tvm_runtime::load_param_dict(params_bytes2).unwrap();
    let graph2 = tvm_runtime::Graph::try_from(graph_json2).unwrap();
    let mut exec2 = tvm_runtime::GraphExecutor::new(graph2, &syslib).unwrap();
    exec2.load_params(params2);
    let listener = std::net::TcpListener::bind("127.0.0.1:4242").unwrap();
    // for stream in listener.incoming() {
    //     let mut stream = stream.unwrap();
    //     if let Err(_) =
    //         stream.read_exact(exec.get_input("data").unwrap().data().view().as_mut_slice())
    //     {
    //         continue;
    //     }
    //     exec.run();
    //     if let Err(_) = stream.write_all(exec.get_output(0).unwrap().data().as_slice()) {
    //         continue;
    //     }
    // }
    let mut stream = listener.accept().unwrap().0;
    stream.read_exact(exec.get_input("data").unwrap().data().view().as_mut_slice()).unwrap();
    exec.run();
    stream.write_all(exec.get_output(0).unwrap().data().as_slice()).unwrap();

    let mut stream1 = listener.accept().unwrap().0;
    stream1.read_exact(exec1.get_input("data").unwrap().data().view().as_mut_slice()).unwrap();
    exec1.run();
    stream1.write_all(exec1.get_output(0).unwrap().data().as_slice()).unwrap();

    let mut stream2 = listener.accept().unwrap().0;
    stream2.read_exact(exec2.get_input("data").unwrap().data().view().as_mut_slice()).unwrap();
    exec2.run();
    stream2.write_all(exec2.get_output(0).unwrap().data().as_slice()).unwrap();
}
