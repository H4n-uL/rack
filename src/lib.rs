//!                                   rack                                   !//
//!
//! Crafted by HaƞuL in 2025
//! Description: Range coder for stream compression
//! Licence: Non-assertion

#![no_std]

extern crate alloc;

use alloc::{
    boxed::Box,
    collections::VecDeque,
    vec::Vec
};

const BYTE_VALS: usize = 1 << u8::BITS; // 256 possible byte values
const SYM_COUNT: usize = BYTE_VALS + 2; // 256 byte values + CLEAR + FLUSH

const CLEAR: usize = BYTE_VALS;     // CLEAR symbol, resets the probability model
const FLUSH: usize = BYTE_VALS + 1; // FLUSH symbol, forces encoder to flush and reset state

const TOP: u32 = 1 << (u32::BITS - u8::BITS);  // 0x01000000
const BOT: u32 = TOP >> u8::BITS;              // 0x00010000
const PROB_SUM_MAX: u32 = 0x1000;
const PROB_INIT: [u32; SYM_COUNT] = [1; SYM_COUNT];

pub struct Rack {
    low: u32,
    range: u32,
    prob: Box<[u32; SYM_COUNT]>
}

impl Rack {
    pub fn new() -> Self {
        return Self {
            low: 0,
            range: u32::MAX,
            prob: Box::new(PROB_INIT),
        };
    }

    fn encsym(&mut self, sym: usize, buf: &mut Vec<u8>) {
        let total = self.prob.iter().sum::<u32>();
        let prob_sum = self.prob[0..sym].iter().sum::<u32>();

        self.range /= total;
        self.low = self.low.wrapping_add(prob_sum * self.range);
        self.range *= self.prob[sym];

        loop {
            if (self.low ^ self.low.wrapping_add(self.range)) >= TOP {
                if self.range >= BOT { break; }
                self.range = self.low.wrapping_neg() & (BOT - 1);
            }
            buf.push((self.low >> (u32::BITS - u8::BITS)) as u8);
            self.range <<= u8::BITS;
            self.low <<= u8::BITS;
        }

        match sym {
            0..BYTE_VALS => {
                self.prob[sym] = self.prob[sym].saturating_add(1);

                let total = self.prob.iter().sum::<u32>();
                if total > PROB_SUM_MAX {
                    for p in self.prob.iter_mut() {
                        *p = (*p >> 1).max(1);
                    }
                }
            }
            CLEAR => {
                *self.prob = PROB_INIT;
            }
            FLUSH => {
                for _ in 0..size_of::<u32>() {
                    buf.push((self.low >> (u32::BITS - u8::BITS)) as u8);
                    self.low <<= u8::BITS;
                }
                self.range = u32::MAX;
            }
            _ => unreachable!()
        }
    }

    pub fn proc(&mut self, data: &[u8]) -> Vec<u8> {
        let mut buf = Vec::new();

        for &byte in data {
            self.encsym(byte as usize, &mut buf);
        }

        return buf;
    }

    pub fn flush(&mut self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.encsym(FLUSH, &mut buf);
        return buf;
    }

    pub fn clear(&mut self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.encsym(CLEAR, &mut buf);
        return buf;
    }

    pub fn finish(&mut self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.encsym(CLEAR, &mut buf);
        self.encsym(FLUSH, &mut buf);
        return buf;
    }
}

pub struct Unrack {
    low: u32,
    range: u32,
    prob: Box<[u32; SYM_COUNT]>,
    inbuf: VecDeque<u8>
}

impl Unrack {
    pub fn new() -> Self {
        return Self {
            low: 0,
            range: u32::MAX,
            prob: Box::new(PROB_INIT),
            inbuf: VecDeque::new(),
        };
    }

    fn code(&self) -> Option<u32> {
        if self.inbuf.len() < size_of::<u32>() {
            return None;
        }

        let mut bytes = [0u8; size_of::<u32>()];
        for (i, b) in bytes.iter_mut().enumerate() {
            *b = self.inbuf[i];
        }

        return Some(u32::from_be_bytes(bytes));
    }

    fn pop(&mut self) {
        self.inbuf.pop_front();
    }

    fn decsym(&mut self) -> Option<usize> {
        let code = self.code()?;
        let total = self.prob.iter().sum::<u32>();

        self.range /= total;
        let count = code.wrapping_sub(self.low) / self.range;

        let mut acc = 0u32;
        let mut sym = 0usize;
        loop {
            acc += self.prob[sym];
            if count < acc { break; }
            sym += 1;
            if sym >= self.prob.len() { return None; }
        }

        let prob_sum = acc - self.prob[sym];
        self.low = self.low.wrapping_add(prob_sum * self.range);
        self.range *= self.prob[sym];

        loop {
            if (self.low ^ self.low.wrapping_add(self.range)) >= TOP {
                if self.range >= BOT { break; }
                self.range = self.low.wrapping_neg() & (BOT - 1);
            }
            self.pop();
            self.range <<= 8;
            self.low <<= 8;
        }

        match sym {
            0..BYTE_VALS => {
                self.prob[sym] = self.prob[sym].saturating_add(1);
                let total = self.prob.iter().sum::<u32>();
                if total > PROB_SUM_MAX {
                    for p in self.prob.iter_mut() {
                        *p = (*p >> 1).max(1);
                    }
                }
            }
            CLEAR => {
                *self.prob = PROB_INIT;
            }
            FLUSH => {
                for _ in 0..size_of::<u32>() {
                    self.pop();
                }
                self.low = 0;
                self.range = u32::MAX;
            }
            _ => unreachable!()
        }

        return Some(sym);
    }

    pub fn proc(&mut self, data: &[u8]) -> Vec<u8> {
        self.inbuf.extend(data.iter());
        let mut buf = Vec::new();

        while let Some(sym) = self.decsym() {
            if sym < BYTE_VALS {
                buf.push(sym as u8);
            }
        }

        return buf;
    }
}
