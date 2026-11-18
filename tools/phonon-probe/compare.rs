//! Compare native-reference tensor traces; no interpreter or model conversion.
use std::{fs,path::Path};
fn floats(path:&Path)->Vec<f32>{let b=fs::read(path).unwrap();assert_eq!(b.len()%4,0);b.chunks_exact(4).map(|v|f32::from_le_bytes(v.try_into().unwrap())).collect()}
fn main(){
    let args:Vec<_>=std::env::args().skip(1).collect();assert_eq!(args.len(),2);
    let mut files:Vec<_>=fs::read_dir(&args[0]).unwrap().map(|e|e.unwrap().path()).filter(|p|p.extension().is_some_and(|s|s=="f32")).collect();files.sort();
    for p in files {
        let name=p.file_name().unwrap();let a=floats(&p);let b=floats(&Path::new(&args[1]).join(name));assert_eq!(a.len(),b.len(),"{:?}",name);
        let mut max=0f32;let mut squared=0f64;let mut norm=0f64;
        for (a,b) in a.iter().zip(&b){assert!(a.is_finite()&&b.is_finite());max=max.max((a-b).abs());squared+=(*a as f64-*b as f64).powi(2);norm+=(*a as f64).powi(2);}
        println!("{}: {} floats; max_abs {:.8}; rms {:.8}; relative_l2 {:.8}",name.to_string_lossy(),a.len(),max,(squared/a.len() as f64).sqrt(),(squared/norm.max(1e-30)).sqrt());
    }
}
