use super::*;
use crate::material::strategies::*;
use crate::material::traits::*;
use crate::material::types::*;
use crate::material::ReferenceMaterial;
use crate::mesh::generators_2d::*;
use crate::mesh::generators_3d::*;
use crate::mesh::types::*;
use crate::solver::*;
use std::path::Path;
use std::fs::{create_dir_all, File};
use std::io::Write;


#[test]
    #[ignore]
    fn test_execute_mini_paper_parametric_campaign() {
        let result_markdown = run_mini_paper_parametric_campaign();
        println!("\n{}", result_markdown);
        assert!(result_markdown.len() > 500);
    }
