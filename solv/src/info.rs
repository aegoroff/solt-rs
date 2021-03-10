use crate::ConsumeDisplay;
use ansi_term::Colour::{Red, RGB};
use prettytable::format::TableFormat;
use prettytable::{format, Table};
use solp::ast::Solution;
use solp::{msbuild, Consume};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fmt::Display;

pub struct Info {
    debug: bool,
    total_projects: BTreeMap<String, i32>,
    projects_in_solutions: BTreeMap<String, i32>,
    solutions: i32,
}

impl Info {
    pub fn new_box(debug: bool) -> Box<dyn ConsumeDisplay> {
        Box::new(Self {
            debug,
            total_projects: BTreeMap::new(),
            projects_in_solutions: BTreeMap::new(),
            solutions: 0,
        })
    }

    pub fn new_format() -> TableFormat {
        format::FormatBuilder::new()
            .column_separator(' ')
            .borders(' ')
            .separators(
                &[format::LinePosition::Title],
                format::LineSeparator::new('-', ' ', ' ', ' '),
            )
            .indent(3)
            .padding(0, 0)
            .build()
    }

    pub fn print_one_column_table(head: &str, set: BTreeSet<&str>) {
        if set.is_empty() {
            return;
        }
        let mut table = Table::new();

        let fmt = Info::new_format();
        table.set_format(fmt);
        table.set_titles(row![bF=> head]);

        for item in set.iter() {
            table.add_row(row![*item]);
        }

        table.printstd();
        println!();
    }

    pub fn err(debug: bool, path: &str) {
        if debug {
            return;
        }
        let path = Red.paint(path);
        eprintln!("Error parsing {} solution", path);
    }
}

impl Consume for Info {
    fn ok(&mut self, path: &str, solution: &Solution) {
        self.solutions += 1;
        let mut projects_by_type: BTreeMap<&str, i32> = BTreeMap::new();
        for prj in &solution.projects {
            if msbuild::is_solution_folder(prj.type_id) {
                continue;
            }
            *projects_by_type.entry(prj.type_descr).or_insert(0) += 1;
        }

        let path = RGB(0xAA, 0xAA, 0xAA).paint(path);
        println!(" {}", path);

        let mut table = Table::new();

        let fmt = format::FormatBuilder::new()
            .column_separator(' ')
            .borders(' ')
            .indent(0)
            .padding(1, 0)
            .build();
        table.set_format(fmt);

        table.add_row(row!["Format", bF->solution.format]);
        table.add_row(row!["Product", bF->solution.product]);

        for version in &solution.versions {
            table.add_row(row![version.name, bF->version.ver]);
        }
        table.printstd();

        println!();

        let mut table = Table::new();

        let fmt = Info::new_format();
        table.set_format(fmt);
        table.set_titles(row![bF=> "Project type", "Count"]);

        for (key, value) in projects_by_type.iter() {
            *self.total_projects.entry(String::from(*key)).or_insert(0) += *value;
            *self
                .projects_in_solutions
                .entry(String::from(*key))
                .or_insert(0) += 1;
            table.add_row(row![*key, bFg->*value]);
        }

        table.printstd();
        println!();

        let configurations = solution
            .solution_configs
            .iter()
            .map(|c| c.config)
            .collect::<BTreeSet<&str>>();

        let platforms = solution
            .solution_configs
            .iter()
            .map(|c| c.platform)
            .collect::<BTreeSet<&str>>();

        Info::print_one_column_table("Configuration", configurations);
        Info::print_one_column_table("Platform", platforms);
    }

    fn err(&self, path: &str) {
        Info::err(self.debug, path);
    }

    fn is_debug(&self) -> bool {
        self.debug
    }
}

impl Display for Info {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", Red.bold().paint(" Totals:"))?;
        writeln!(f, "")?;

        let mut table = Table::new();

        let fmt = Info::new_format();
        table.set_format(fmt);
        table
            .set_titles(row![bF->"Project type", bF->"Count", cbF->"%", bF->"Solutions", cbF->"%"]);

        let projects = self.total_projects.iter().fold(0, |total, p| total + *p.1);

        for (key, value) in self.total_projects.iter() {
            let p = (*value as f64 / projects as f64) * 100 as f64;
            let in_sols = self.projects_in_solutions.get(key).unwrap();
            let ps = (*in_sols as f64 / self.solutions as f64) * 100 as f64;
            table.add_row(row![
                key,
                *value,
                format!("{:.2}%", p),
                r->*in_sols,
                format!("{:.2}%", ps)
            ]);
        }
        table.printstd();
        writeln!(f, "")
    }
}
