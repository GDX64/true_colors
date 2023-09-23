#![feature(iter_partition_in_place)]
use core::fmt;
use std::fmt::Debug;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run_image(img: Vec<u8>) -> Vec<usize> {
    let hsls = RGB::arr_from_byte_array(img)
        .into_iter()
        .map(|rgb| rgb.to_hsl())
        .collect();

    let histogram = HSL::histogram(hsls);
    histogram
}

#[wasm_bindgen]
pub fn calc_palette(img: Vec<u8>, divisions: usize) -> Vec<u32> {
    let arr = RGB::arr_from_byte_array(img);
    let palette = RGB::calc_palette(arr, divisions);
    palette.into_iter().map(|x| x.to_u32()).collect()
}

#[derive(Clone)]
struct RGB {
    r: u8,
    g: u8,
    b: u8,
    index: usize,
}

impl Debug for RGB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //hexadecimal string
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

impl RGB {
    fn black() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            index: 0,
        }
    }

    fn white() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 255,
            index: 0,
        }
    }

    fn to_u32(&self) -> u32 {
        (self.r as u32) << 16 | (self.g as u32) << 8 | self.b as u32
    }

    fn arr_from_byte_array(v: Vec<u8>) -> Vec<RGB> {
        v.chunks_exact(4)
            .map(|chunk| {
                RGB {
                    r: chunk[0],
                    g: chunk[1],
                    b: chunk[2],
                    index: 0,
                }
            })
            .collect()
    }

    fn minus(&self, other: &Self) -> Self {
        Self {
            r: self.r - other.r,
            g: self.g - other.g,
            b: self.b - other.b,
            index: 0,
        }
    }

    fn from_u32(x: u32) -> Self {
        Self {
            r: (x >> 16) as u8,
            g: (x >> 8) as u8,
            b: x as u8,
            index: 0,
        }
    }

    fn calc_palette(v: Vec<RGB>, divisions: usize) -> Vec<RGB> {
        let vs = median_cut(v, divisions);
        vs.into_iter()
            .map(|v| {
                let mut acc = (0u64, 0u64, 0u64);
                let n = v.len();
                v.into_iter().for_each(|x| {
                    acc.0 += x.r as u64;
                    acc.1 += x.g as u64;
                    acc.2 += x.b as u64;
                });
                RGB {
                    r: (acc.0 / n as u64) as u8,
                    g: (acc.1 / n as u64) as u8,
                    b: (acc.2 / n as u64) as u8,
                    index: 0,
                }
            })
            .collect()
    }

    fn to_hsl(&self) -> HSL {
        let r = self.r as f64 / 255.0;
        let g = self.g as f64 / 255.0;
        let b = self.b as f64 / 255.0;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let mut h = (max + min) / 2.0;
        let mut s = h;
        let l = h;
        if max == min {
            h = 0.0;
            s = 0.0;
        } else {
            let d = max - min;
            s = if l > 0.5 {
                d / (2.0 - max - min)
            } else {
                d / (max + min)
            };
            if max == r {
                h = (g - b) / d + if g < b { 6.0 } else { 0.0 };
            } else if max == g {
                h = (b - r) / d + 2.0;
            } else if max == b {
                h = (r - g) / d + 4.0;
            }
            h /= 6.0;
        }
        HSL {
            h: h * 360.0,
            s: s * 100.0,
            l: l * 100.0,
        }
    }
}

struct HSL {
    h: f64,
    s: f64,
    l: f64,
}

impl HSL {
    fn bucket(&self) -> usize {
        self.h.floor() as usize
    }

    fn histogram(HSLs: Vec<HSL>) -> Vec<usize> {
        const N: usize = 360;
        let mut histogram = vec![0; N];
        HSLs.iter().for_each(|x| {
            histogram[x.bucket() % N] += 1;
        });
        histogram
    }
}

#[derive(Debug)]
enum QuadNode<T> {
    Empty,
    Leaf(T),
    Branch(Box<[QuadTree<T>; 4]>),
}

#[derive(Debug)]
struct QuadTree<T> {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    root: QuadNode<T>,
}

trait CanBeLeaf {
    fn x(&self) -> f64;
    fn y(&self) -> f64;
}

impl<T: CanBeLeaf> QuadTree<T> {
    fn new(x: f64, y: f64, height: f64, width: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
            root: QuadNode::<T>::Empty,
        }
    }

    fn insert(&mut self, value: T) -> Option<T> {
        if self.x > value.x()
            || self.y > value.y()
            || self.x + self.width < value.x()
            || self.y + self.height < value.y()
        {
            return Some(value);
        }
        match self.root {
            QuadNode::Empty => {
                self.root = QuadNode::Leaf(value);
            }
            QuadNode::Leaf(_) => {
                let mut nodes = [
                    QuadTree::new(self.x, self.y, self.width / 2.0, self.height / 2.0),
                    QuadTree::new(
                        self.x + self.width / 2.0,
                        self.y,
                        self.width / 2.0,
                        self.height / 2.0,
                    ),
                    QuadTree::new(
                        self.x,
                        self.y + self.height / 2.0,
                        self.width / 2.0,
                        self.height / 2.0,
                    ),
                    QuadTree::new(
                        self.x + self.width / 2.0,
                        self.y + self.height / 2.0,
                        self.width / 2.0,
                        self.height / 2.0,
                    ),
                ];
                Self::try_insert_on_nodes(&mut nodes, value);
                self.root = QuadNode::Branch(Box::new(nodes));
            }
            QuadNode::Branch(ref mut nodes) => {
                Self::try_insert_on_nodes(nodes, value);
            }
        };
        None
    }

    fn try_insert_on_nodes(nodes: &mut [QuadTree<T>; 4], value: T) {
        nodes[0]
            .insert(value)
            .and_then(|v| nodes[1].insert(v))
            .and_then(|v| nodes[2].insert(v))
            .and_then(|v| nodes[3].insert(v));
    }
}

#[derive(Debug)]
struct Point {
    x: f64,
    y: f64,
}

impl CanBeLeaf for Point {
    fn x(&self) -> f64 {
        self.x
    }

    fn y(&self) -> f64 {
        self.y
    }
}

mod test {
    #[test]
    fn test_quad() {
        use super::*;
        let mut tree = QuadTree::new(0.0, 0.0, 100.0, 100.0);
        tree.insert(Point { x: 10.0, y: 10.0 });
        tree.insert(Point { x: 10.0, y: 60.0 });
        println!("{tree:?}")
    }
}

fn median_cut(mut rgb: Vec<RGB>, divisions: usize) -> Vec<Vec<RGB>> {
    if divisions == 0 || rgb.len() <= 1 {
        return vec![rgb];
    }
    let mut min_max = (RGB::white(), RGB::black());
    rgb.iter().for_each(|x| {
        min_max.0.r = min_max.0.r.min(x.r);
        min_max.0.g = min_max.0.g.min(x.g);
        min_max.0.b = min_max.0.b.min(x.b);
        min_max.1.r = min_max.1.r.max(x.r);
        min_max.1.g = min_max.1.g.max(x.g);
        min_max.1.b = min_max.1.b.max(x.b);
    });
    let ranges = min_max.1.minus(&min_max.0);
    let partitions = {
        let red_is_max = ranges.r > ranges.g && ranges.r > ranges.b;
        let green_is_max = ranges.g > ranges.r && ranges.g > ranges.b;
        if red_is_max {
            let red_median = median_of_array(&mut rgb, |item, pivot| item.r < pivot.r).clone();
            let partitions: (Vec<RGB>, Vec<RGB>) = rgb.into_iter().partition(|color| {
                return color.r > red_median.r;
            });
            partitions
        } else if green_is_max {
            let green_median = median_of_array(&mut rgb, |item, pivot| item.g < pivot.g).clone();
            let partitions: (Vec<RGB>, Vec<RGB>) = rgb.into_iter().partition(|color| {
                return color.g > green_median.g;
            });
            partitions
        } else {
            let blue_median = median_of_array(&mut rgb, |item, pivot| item.b < pivot.b).clone();
            let partitions: (Vec<RGB>, Vec<RGB>) = rgb.into_iter().partition(|color| {
                return color.b > blue_median.b;
            });
            partitions
        }
    };
    let mut v1 = median_cut(partitions.0, divisions - 1);
    v1.append(&mut median_cut(partitions.1, divisions - 1));
    v1.retain(|v| v.len() > 0);
    v1
}

fn median_of_array<T, F: Fn(&T, &T) -> bool>(v: &mut [T], is_pivot_bigger: F) -> &T {
    let mut l = 0;
    let mut r = v.len() - 1;
    let k = v.len() / 2;
    loop {
        let mut i = l;
        for j in l..r {
            let pivot = &v[r];
            let pivot_is_bigger = is_pivot_bigger(pivot, &v[j]);
            if pivot_is_bigger {
                v.swap(i, j);
                i += 1;
            }
        }
        v.swap(i, r);
        if i == k {
            return &v[i];
        } else if i < k {
            l = i + 1;
        } else {
            r = i - 1;
        }
    }
}

mod test_median {
    use crate::RGB;

    #[test]
    fn test() {
        let v = vec![0x440044, 0x111100, 0x222200, 0x003333]
            .into_iter()
            .map(RGB::from_u32)
            .collect::<Vec<_>>();
        let v = RGB::calc_palette(v, 2);
        println!("{:?}", v);
    }
}
