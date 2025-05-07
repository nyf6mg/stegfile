use rand::Rng;
use sha256::digest;
use std::fs::*;
use std::io::prelude::*;
use std::{env, process::exit};

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("No arguments were supplied. Use \"stegfile -h\" to get more information.");
        exit(1)
    }

    if &args[1] == "-h" || &args[1] == "--help" {
        println!("help menu");
    }

    if &args[1] == "-k" || &args[1] == "--key-tool" {
        if args.len() <= 3 {
            println!("To use key-tool, please provide your secret key in hexadecimal format.")
        } else {
            let mut file = File::create("inivec.text")?;
            let mut ind = 0;
            let mut key = args[2].clone();
            while ind < args[3].parse().unwrap_or(0) {
                file.write_all(key.as_bytes())?;
                file.write_all(b"\n")?;
                key = digest(key);
                ind += 1;
            }
        }
        return Ok(());
    }

    if &args[1] == "-i" || &args[1] == "--init" {
        if args.len() <= 2 {
            println!("To initiallize, please provide the length of keys you are using in bits, and the maximum number of files to put in the system.");
        } else {
            let key_len = args[2].parse::<i32>().unwrap();
            std::fs::create_dir("./files")?;
            for x in 0..key_len {
                let mut file = File::create(format!("./files/{}", x))?;
                let mut rng = rand::rng();
                let random_data: Vec<u8> = (0..key_len / 8).map(|_| rng.random()).collect();

                file.write_all(&random_data)?;
            }
            return Ok(());
        }
    }

    if &args[1] == "-e" || &args[1] == "--encode" {
        if args.len() <= 4 {
            println!("To encode a file, please provide the filename, and the key you are using, and what level you are encoding the file on.");
        } else {
            let key_array = create_key_array(args[3].clone(), args[4].parse().unwrap_or(0));
            let ortho_keys = reverse_gm_ortho(key_array);
            let k = ortho_keys[0].clone();
            let mut index = 0;
            let mut fin = 0;
            let mut first = true;
            let mut working: Vec<u8> = Vec::new();
            for bit in k {
                if bit == 1 {
                    if first {
                        fin = index;
                        first = false;
                        let mut file = File::open(format!("./files/{}", index))?;
                        let mut contents = Vec::new();
                        file.read_to_end(&mut contents)?;
                        working = contents;
                    } else {
                        let mut file = File::open(format!("./files/{}", index))?;
                        let mut contents = Vec::new();
                        file.read_to_end(&mut contents)?;
                        working = working
                            .iter()
                            .zip(contents.iter())
                            .map(|(x, y)| x ^ y)
                            .collect();
                    }
                }
                index += 1;
            }
            let mut user_file = File::open(args[2].clone())?;
            let mut contents = Vec::new();
            user_file.read_to_end(&mut contents)?;
            while contents.len() < 256 {
                contents.push(0xff);
            }
            working = working
                .iter()
                .zip(contents.iter())
                .map(|(x, y)| x ^ y)
                .collect();
            let mut final_file = File::open(format!("./files/{}", fin))?;
            let mut final_contents = Vec::new();
            final_file.read_to_end(&mut final_contents)?;
            working = working
                .iter()
                .zip(final_contents.iter())
                .map(|(x, y)| x ^ y)
                .collect();
            std::fs::remove_file(format!("./files/{}", fin))?;
            let mut file = File::create(format!("./files/{}", fin))?;
            file.write_all(&working)?;
        }
    }

    if args[1] == "-d" || args[1] == "--decode" {
        if args.len() <= 3 {
            println!("To dencode a file, please provide the key you are using, and what level the file is on.");
        } else {
            let key_array = create_key_array(args[2].clone(), args[3].parse().unwrap_or(0));
            let ortho_keys = reverse_gm_ortho(key_array);
            let k = ortho_keys[0].clone();
            let mut index = 0;
            let mut working: Vec<u8> = Vec::new();
            let mut first = true;
            for bit in k {
                if bit == 1 {
                    if first {
                        first = false;
                        let mut file = File::open(format!("./files/{}", index))?;
                        let mut contents = Vec::new();
                        file.read_to_end(&mut contents)?;
                        working = contents;
                    } else {
                        let mut file = File::open(format!("./files/{}", index))?;
                        let mut contents = Vec::new();
                        file.read_to_end(&mut contents)?;
                        working = working
                            .iter()
                            .zip(contents.iter())
                            .map(|(x, y)| x ^ y)
                            .collect();
                    }
                }
                index += 1;
            }
            let mut file = File::create("./user_file.txt")?;
            file.write_all(&working)?;
        }
    }

    Ok(())
}

fn reverse_gm_ortho(binary_vec: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    let n = binary_vec.len();
    let mut ks: Vec<Vec<u8>> = vec![vec![0; binary_vec[0].len()]; n];

    for (i, v) in binary_vec.iter().enumerate() {
        ks[i] = v.clone();
    }

    for i in (0..n).rev() {
        for j in (i + 1)..n {
            let dot: u8 = ks[i]
                .iter()
                .zip(ks[j].iter())
                .map(|(a, b)| a & b)
                .fold(0, |acc, x| acc ^ x);

            if dot == 1 {
                ks[i] = ks[i].iter().zip(ks[j].iter()).map(|(a, b)| a ^ b).collect();
            }
        }
        let mut count = 0;
        for num in &ks[i] {
            if *num == 1 {
                count += 1;
            }
        }
        if count % 2 == 0 {
            let mut flag = true;
            let mut x = 0;
            while flag {
                if ks[i][x] == 0 {
                    ks[i][x] = 1;
                    flag = false;
                }
                x += 1;
            }
        }
    }

    ks
}

fn binary_array_from_hex(hex: String) -> Vec<u8> {
    let byte_array: Result<Vec<u8>, _> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect();
    let bytes = match byte_array {
        Ok(my_bytes) => my_bytes,
        Err(_e) => {
            println!("early exit for some reason");
            exit(1)
        }
    };
    let mut binary_array: Vec<u8> = Vec::new();
    for byte in bytes {
        for bit_index in (0..8).rev() {
            let bit = (byte >> bit_index) & 1;
            binary_array.push(bit);
        }
    }
    binary_array
}

fn create_key_array(given_key: String, len: u8) -> Vec<Vec<u8>> {
    let mut key_array: Vec<Vec<u8>> = Vec::new();
    let mut key = given_key.clone();
    let mut first_key = binary_array_from_hex(key.clone());
    let mut count = 0;
    for num in &first_key {
        if *num == 1 {
            count += 1;
        }
    }
    if count % 2 == 0 {
        let mut flag = true;
        let mut i = 0;
        while flag {
            if first_key[i] == 0 {
                first_key[i] = 1;
                flag = false;
            }
            i += 1;
        }
    }
    key_array.push(first_key);
    for _x in 1..len {
        key = digest(key);
        let mut next_k = binary_array_from_hex(key.clone());
        let mut count = 0;
        for num in &next_k {
            if *num == 1 {
                count += 1;
            }
        }
        if count % 2 == 0 {
            let mut flag = true;
            let mut i = 0;
            while flag {
                if next_k[i] == 0 {
                    next_k[i] = 1;
                    flag = false;
                }
                i += 1;
            }
        }
        key_array.push(next_k);
    }
    key_array
}
