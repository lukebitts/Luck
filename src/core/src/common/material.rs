#![allow(missing_docs)]

use glium::Program;
use glium::uniforms::{UniformValue};
use std::rc::Rc;

/// This will likely be refactored, so I'll leave the documentation blank.
#[derive(Clone)]
pub struct Material {
    program: Rc<Program>,
    uniforms: Vec<(String, UniformValue<'static>)>
}

impl Material {
    pub fn new(program: Rc<Program>, uniforms: Vec<(String, UniformValue<'static>)>) -> Self {
        Material {
            program: program,
            uniforms: uniforms
        }
    }
    pub fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&self, f: &mut F) {
        for pair in &self.uniforms {
            f(&pair.0[..], pair.1);
        }
    }
    pub fn set_uniform(&mut self, key: &str, value: UniformValue<'static>) {
        let mut index = -1i32;

        for (i, v) in self.uniforms.iter().enumerate() {
            if v.0 == key {
                index = i as i32;
            }
        }

        if index > 0 {
            self.uniforms[index as usize] = (key.to_owned(), value);
        }
        else {
            self.uniforms.push((key.to_owned(), value));
        }
    }
}
