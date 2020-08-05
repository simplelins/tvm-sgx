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
use std::process::Command;
use std::io::prelude::*;

macro_rules! mf_dir {
    ($p:literal) => {
        concat!(env!("CARGO_MANIFEST_DIR"), $p)
    };
}

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let build_output = Command::new(mf_dir!("/src/build_model.py"))
        .arg(&out_dir)
        .env(
            "PYTHONPATH",
            concat!(
                mf_dir!("/../../python"),
                ":",
                mf_dir!("/../../nnvm/python"),
                ":",
                mf_dir!("/../../topi/python")
            ),
        )
        .output()
        .expect("Failed to build model");
    assert!(
        ["model.o", "graph.json", "params.bin"]
            .iter()
            .all(|f| { std::path::Path::new(&format!("{}/{}", out_dir, f)).exists() }),
        "Could not build tvm lib: STDOUT:\n\n{}\n\nSTDERR\n\n{}",
        String::from_utf8(build_output.stdout).unwrap().trim(),
        String::from_utf8(build_output.stderr).unwrap().trim()
    );

    let sysroot_output = Command::new("rustc")
        .args(&["--print", "sysroot"])
        .output()
        .expect("Failed to get sysroot");
    let sysroot = String::from_utf8(sysroot_output.stdout).unwrap();
    let sysroot = sysroot.trim();
    let mut llvm_tools_path = std::path::PathBuf::from(&sysroot);
    llvm_tools_path.push("lib/rustlib/x86_64-unknown-linux-gnu/bin");

    Command::new("rustup")
        .args(&["component", "add", "llvm-tools-preview"])
        .output()
        .expect("failed to install llvm tools");

    std::process::Command::new(llvm_tools_path.join("llvm-objcopy"))
        .arg("--globalize-symbol=__tvm_module_startup")
        .arg("--remove-section=.ctors")
        .arg(&format!("{}/model.o", out_dir))
        .output()
        .expect("gould not gloablize startup function");

    std::process::Command::new(llvm_tools_path.join("llvm-ar"))
        .arg("rcs")
        .arg(&format!("{}/libmodel.a", out_dir))
        .arg(&format!("{}/model.o", out_dir))
        .output()
        .expect("failed to package model archive");
        

    let model1_nm = std::process::Command::new(llvm_tools_path.join("llvm-nm"))
        .arg("--defined-only")
        .arg(&format!("{}/model1.o", out_dir))
        .output()
        .expect("nm function");

    let mut file = std::fs::File::create(&format!("{}/model1_name", out_dir)).expect("create failed");
    let lines_string = String::from_utf8_lossy(&model1_nm.stdout);
    for line in lines_string.split("\n") {
        match line.split_whitespace().collect::<Vec<&str>>().pop() {
            Some(name) => {
                writeln!(&mut file, "{0}  model1_{0}", name).expect("write failed");
            }
            None => continue,
        }
    }
            
    std::process::Command::new(llvm_tools_path.join("llvm-objcopy"))
        .arg("--globalize-symbol=__tvm_module_startup")
        .arg(&format!("--redefine-syms={}/model1_name", out_dir))
        .arg("--remove-section=.ctors")
        .arg(&format!("{}/model1.o", out_dir))
        .output()
        .expect("gould not gloablize startup function");

    std::process::Command::new(llvm_tools_path.join("llvm-ar"))
        .arg("rcs")
        .arg(&format!("{}/libmodel1.a", out_dir))
        .arg(&format!("{}/model1.o", out_dir))
        .output()
        .expect("failed to package model archive");
    let model2_nm = std::process::Command::new(llvm_tools_path.join("llvm-nm"))
        .arg("--defined-only")
        .arg(&format!("{}/model2.o", out_dir))
        .output()
        .expect("nm function");
    let mut file = std::fs::File::create(&format!("{}/model2_name", out_dir)).expect("create failed");
    let lines_string = String::from_utf8_lossy(&model2_nm.stdout);
        for line in lines_string.split("\n") {
            match line.split_whitespace().collect::<Vec<&str>>().pop() {
                Some(name) => {
                    writeln!(&mut file, "{0}  model2_{0}", name).expect("write failed");
                }
                None => continue,
            }
        }
    std::process::Command::new(llvm_tools_path.join("llvm-objcopy"))
        .arg("--globalize-symbol=__tvm_module_startup")
        .arg(&format!("--redefine-syms={}/model2_name", out_dir))
        .arg("--remove-section=.ctors")
        .arg(&format!("{}/model2.o", out_dir))
        .output()
        .expect("gould not gloablize startup function");
        
    std::process::Command::new(llvm_tools_path.join("llvm-ar"))
        .arg("rcs")
        .arg(&format!("{}/libmodel2.a", out_dir))
        .arg(&format!("{}/model2.o", out_dir))
        .output()
        .expect("failed to package model archive");

    println!("cargo:rustc-link-lib=static=model2");
    println!("cargo:rustc-link-lib=static=model1");
    println!("cargo:rustc-link-lib=static=model");
    println!("cargo:rustc-link-search=native={}", out_dir);
}
