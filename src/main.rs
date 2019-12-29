use image as im;
use rayon::prelude::*;
use std::{sync, time};

type Pix = im::Rgb<u8>;

const SIZE: usize = 1024;
const DEPTH: usize = 4096;
const THREAD_CNT: usize = 16;

const VIEWS: [(f32, f32, f32); 4] = [
    (-0.7463, 0.1102, 0.005 as f32 / 2f32),
    (-0.745428, 0.113009, 3e-5 as f32 / 2f32),
    (-0.748, 0.1, 0.014),
    (-0.745428, 0.113009, 3.0E-5 / 2f32),
];
// struct PixSeq {
//     curr: usize,
// }

// impl PixSeq {

// }

// impl Iterator for PixSeq {
//     type Item = (u32, u32);
//     fn next(&mut self) -> Option<Self::Item> {
//         Some((0, 0))
//     }
// }

fn interpolate(val: f32, orig_range: (f32, f32), res_range: (f32, f32)) -> f32 {
    ((val - orig_range.0) / (orig_range.1 - orig_range.0))
        * (res_range.1 - res_range.0)
        + res_range.0
}

fn v_2_color(v: f32) -> Pix {
    let cl = interpolate(v, (0., 1.), (0., 255.)) as u8;
    // let red = match 0.5 - v {
    //     v if v >= 0.5 || v <= 0. => 0.,
    //     v => interpolate(v, (0., 0.5), (0., 255.))
    // } as u8;
    // let green = match v {
    //     v if v > 0.5 => interpolate(1. - v, (0., 0.5), (0., 255.)),
    //     v => interpolate(v, (0., 0.5), (0., 255.))
    // } as u8;
    // let blue = match v {
    //     v if v > 0.5 => interpolate(v - 0.5, (0., 0.5), (0., 255.)),
    //     _ => 0.
    // } as u8;
    im::Rgb([cl, cl, cl])
}

fn guess_pixel(x: u32, y: u32) -> Pix {
    let (cx, cy, w) = VIEWS[1];
    let x = interpolate(x as f32, (0.0, SIZE as f32), (cx - w, cx + w));
    let y = interpolate(y as f32, (0.0, SIZE as f32), (cy - w, cy + w));
    let mut a: f32 = 0.0;
    let mut b: f32 = 0.0;
    let mut iter = 0;
    loop {
        let t = a.powi(2) - b.powi(2) + x;
        b = 2.0 * a * b + y;
        a = t;
        iter += 1;
        if iter > DEPTH || a.powi(2) + b.powi(2) > 4.0 {
            break;
        }
    }
    match iter {
        i if i >= DEPTH => im::Rgb([0, 0, 0]),
        _ => {
            let interp =
                interpolate(iter as f32, (0.0, DEPTH as f32), (0.0, 1.0));
            v_2_color(interp)
        }
    }
}

fn main() {
    let total_len = SIZE * SIZE;
    let div = THREAD_CNT;
    let slsz = total_len / div; // one piece size
    // let acc = {
    //     let v: Vec<Vec<u8>> = vec![Vec::with_capacity(slsz); div];
    //     let v = sync::Mutex::new(v);
    //     sync::Arc::new(v)
    // };
    // let pic: im::ImageBuffer<Pix, _> =
    //     im::ImageBuffer::new(SIZE as u32, SIZE as u32);
    let mut pic: Vec<(u32, u32, Pix)> =
        vec![(0, 0, im::Rgb([0, 0, 0])); SIZE * SIZE];
    for (idx, px) in pic.iter_mut().enumerate() {
        px.0 = idx as u32 % SIZE as u32;
        px.1 = idx as u32 / SIZE as u32;
    }
    let timer = time::Instant::now();
    pic.par_iter_mut().for_each(|px| {
        px.2 = guess_pixel(px.0, px.1);
    });
    let pic =
        pic.iter()
            .fold(Vec::with_capacity(SIZE * SIZE * 3), |mut acc, c| {
                let p = c.2;
                acc.extend_from_slice(&[p[0], p[1], p[2]]);
                acc
            });
    // crossbeam::scope(|s| {
    //     for i in 0..div {
    //         println!("thread: {}", i);
    //         let acc = acc.clone();
    //         s.spawn(move |_| {
    //             let mut sl: Vec<u8> = Vec::with_capacity(slsz);
    //             let offset = slsz * i;
    //             for d in 0..slsz {
    //                 let (x, y) = ((d + offset) % SIZE, (d + offset) / SIZE);
    //                 let cl = guess_pixel(x as u32, y as u32);
    //                 for j in 0..3 {
    //                     sl.push(cl.0[j])
    //                 }
    //             }
    //             let mut acc = acc.lock().unwrap();
    //             acc[i].extend_from_slice(&sl);
    //         });
    //     }
    // })
    // .unwrap();

    // let acc: Vec<u8> = acc.lock().unwrap().iter().fold(
    //     Vec::with_capacity(total_len),
    //     |mut acc, sl| {
    //         acc.extend_from_slice(&sl);
    //         acc
    //     },
    // );
    // println!("{}", pic.len());
    let pic: im::ImageBuffer<Pix, _> =
        im::ImageBuffer::from_vec(SIZE as u32, SIZE as u32, pic)
            .unwrap();

    println!("Calculating time: {}", timer.elapsed().as_millis());
    pic.save("res2.png").expect("Failed to save the image");
}
