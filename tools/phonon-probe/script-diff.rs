//! Script disagreement only: the written script is not verified spoken truth.
use std::{env, fs};
fn words(s: &str) -> Vec<String> {
    s.split_whitespace().map(|w| w.chars().filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase).collect::<String>()).filter(|w| !w.is_empty()).collect()
}
fn main() {
    let paths: Vec<_> = env::args().skip(1).collect();
    assert!(paths.len() >= 2, "script-diff REFERENCE HYPOTHESIS...");
    let a = words(&fs::read_to_string(&paths[0]).unwrap());
    for path in &paths[1..] {
        let b = words(&fs::read_to_string(path).unwrap());
        let n = b.len()+1;
        let mut d = vec![0usize; (a.len()+1)*n];
        for i in 0..=a.len() {d[i*n]=i;}
        for j in 0..=b.len() {d[j]=j;}
        for i in 1..=a.len() { for j in 1..=b.len() {
            d[i*n+j]=(d[(i-1)*n+j-1]+usize::from(a[i-1]!=b[j-1]))
                .min(d[(i-1)*n+j]+1).min(d[i*n+j-1]+1);
        }}
        let (mut i, mut j)=(a.len(),b.len());
        let (mut sub, mut del, mut ins)=(0,0,0);
        let mut differences=Vec::new();
        while i>0 || j>0 {
            if i>0 && j>0 && d[i*n+j]==d[(i-1)*n+j-1]+usize::from(a[i-1]!=b[j-1]) {
                if a[i-1]!=b[j-1] {sub+=1;differences.push(format!("{} -> {}",a[i-1],b[j-1]));}
                i-=1;j-=1;
            } else if i>0 && d[i*n+j]==d[(i-1)*n+j]+1 {
                del+=1;differences.push(format!("{} -> [absent]",a[i-1]));i-=1;
            } else {ins+=1;differences.push(format!("[extra] -> {}",b[j-1]));j-=1;}
        }
        differences.reverse();
        println!("{path}: reference {}, output {}, disagreements {} (S{sub} D{del} I{ins}); UNVERIFIED script comparison", a.len(),b.len(),sub+del+ins);
        for difference in differences {println!("  {difference}");}
    }
}
