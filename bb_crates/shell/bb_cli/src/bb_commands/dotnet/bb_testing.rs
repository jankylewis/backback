//! `bb test list` and `bb test run`: the headless engine on real projects.
//! Never prints a runsettings secret: the summary counts them.

use std::path::{Path, PathBuf};
use std::time::Instant;

use bb_dotnet::bb_project::RunSettings;
use bb_dotnet::{Capability, ProjectInfo, ProjectPath, Solution, SolutionPath};
use bb_testkit::{
    run as run_engine, ApexBackend, ApexSettings, Category, CommandLine, Configuration,
    DotnetBackend, RunReport, Selection, SpecList, TestBackend, Workspace,
};

use super::scan::runsettings_line;
use super::{tree, TestError};
use crate::bb_commands::CommandError;
use crate::cli::{BackendChoice, TestAction, TestArgs};

/// The backend picked for one project.
enum Engine {
    Apex(ApexBackend),
    Dotnet(DotnetBackend),
}

pub(crate) async fn run(args: &TestArgs) -> Result<(), CommandError> {
    let target: PathBuf =
        std::path::absolute(&args.target).map_err(|source: std::io::Error| CommandError::Read {
            path: args.target.clone(),
            source,
        })?;
    match args.action {
        TestAction::List => list(args, &target).await,
        TestAction::Run => run_one(args, &ProjectPath::from_path(target)).await,
    }
}

async fn list(args: &TestArgs, target: &Path) -> Result<(), CommandError> {
    let projects: Vec<ProjectInfo> = test_projects(target)?;
    let started: Instant = Instant::now();
    let mut failed: usize = 0;
    let mut rows: Vec<(String, &'static str, usize, String)> = Vec::new();
    for info in &projects {
        let engine: Engine = choose(args, info)?;
        let listed: Result<SpecList, bb_testkit::TestkitError> = match &engine {
            Engine::Apex(b) => b.list(&info.path).await,
            Engine::Dotnet(b) => b.list(&info.path).await,
        };
        match listed {
            Ok(list) => {
                tree::listed(&list, name(&engine));
                rows.push((
                    info.name.clone(),
                    name(&engine),
                    list.specs.len(),
                    tree::time(list.elapsed),
                ));
            }
            Err(e) => {
                println!("{}  [{}] ERROR {e}", info.name, name(&engine));
                failed = failed.saturating_add(1);
            }
        }
    }
    for (project, backend, count, took) in &rows {
        println!("listed    : {project} [{backend}] {count} specs in {took}");
    }
    println!(
        "total     : {} projects in {}",
        projects.len(),
        tree::time(started.elapsed())
    );
    match failed {
        0 => Ok(()),
        n => Err(TestError::Lists(n).into()),
    }
}

async fn run_one(args: &TestArgs, project: &ProjectPath) -> Result<(), CommandError> {
    let info: ProjectInfo = ProjectInfo::load(project)?;
    let engine: Engine = choose(args, &info)?;
    let category: Option<Category> = match &args.category {
        Some(c) => Some(
            Category::new(c)
                .ok_or_else(|| TestError::Usage("--category needs a name".to_owned()))?,
        ),
        None => None,
    };
    let selection: Selection = Selection {
        specs: args.specs.clone(),
        category,
    };
    settings_summary(args, &engine, &info);
    let report: RunReport = match &engine {
        Engine::Apex(b) => run_engine(b, project, &selection).await,
        Engine::Dotnet(b) => run_engine(b, project, &selection).await,
    }
    .map_err(TestError::from)?;
    println!(
        "{}  [{}] run {}, exit {:?}",
        info.name,
        name(&engine),
        tree::time(report.elapsed),
        report.exit_code
    );
    tree::ran(&report.leaves);
    let (pass, fail, skip, missing): (usize, usize, usize, usize) = report.counts();
    println!("summary   : {pass} passed, {fail} failed, {skip} skipped, {missing} errored");
    if fail == 0 && missing == 0 {
        Ok(())
    } else {
        Err(TestError::Specs {
            failed: fail,
            errored: missing,
        }
        .into())
    }
}

/// The test projects of a solution, or the one project given.
fn test_projects(target: &Path) -> Result<Vec<ProjectInfo>, CommandError> {
    let is_solution: bool = target
        .extension()
        .is_some_and(|e: &std::ffi::OsStr| e == "sln" || e == "slnx");
    if !is_solution {
        return Ok(vec![ProjectInfo::load(&ProjectPath::from_path(
            target.to_path_buf(),
        ))?]);
    }
    let listing: Solution = Solution::load(&SolutionPath::new(target.to_path_buf())?)?;
    let infos: Vec<ProjectInfo> = listing
        .projects
        .iter()
        .map(ProjectInfo::load)
        .collect::<Result<Vec<ProjectInfo>, bb_dotnet::ProjectError>>()?;
    Ok(infos
        .into_iter()
        .filter(|i: &ProjectInfo| i.is_test)
        .collect())
}

/// Apex when asked, or when an Apex project's workspace sets `apexTestkit.*`
/// (or `--apex-command` is given); plain `dotnet` otherwise.
fn choose(args: &TestArgs, info: &ProjectInfo) -> Result<Engine, CommandError> {
    let workspace: Workspace = Workspace::of(&info.path);
    let settings: ApexSettings = ApexSettings::load(workspace.root()).map_err(TestError::from)?;
    let wants_apex: bool = match args.backend {
        Some(BackendChoice::Apex) => true,
        Some(BackendChoice::Dotnet) => false,
        None => info.has(Capability::Apex) && (settings.declared || args.apex_command.is_some()),
    };
    let configuration: Configuration = match &args.configuration {
        Some(c) => Configuration::new(c)
            .ok_or_else(|| TestError::Usage(format!("bad configuration {c:?}")))?,
        None => settings.configuration.clone(),
    };
    if !wants_apex {
        let backend: DotnetBackend = DotnetBackend::for_project(
            &info.path,
            configuration,
            args.settings.clone(),
            args.build,
        )
        .map_err(TestError::from)?;
        return Ok(Engine::Dotnet(backend));
    }
    let mut chosen: ApexSettings = settings;
    chosen.configuration = configuration;
    chosen.build_before_run = chosen.build_before_run || args.build;
    if let Some(path) = &args.settings {
        chosen.run_settings = Some(path.clone());
    }
    let mut backend: ApexBackend = ApexBackend::new(workspace, chosen).map_err(TestError::from)?;
    if let Some(command) = &args.apex_command {
        let line: CommandLine = CommandLine::parse(command)
            .ok_or_else(|| TestError::Usage("--apex-command is empty".to_owned()))?;
        backend = backend.with_command(line);
    }
    Ok(Engine::Apex(backend))
}

fn name(engine: &Engine) -> &'static str {
    match engine {
        Engine::Apex(b) => b.name(),
        Engine::Dotnet(b) => b.name(),
    }
}

/// The runsettings the run uses: counts and unset names, never values.
fn settings_summary(args: &TestArgs, engine: &Engine, info: &ProjectInfo) {
    let path: Option<PathBuf> = match engine {
        Engine::Apex(b) => b.settings().run_settings.clone(),
        Engine::Dotnet(_) => args.settings.clone(),
    };
    let parsed: Option<RunSettings> = match path {
        Some(p) => std::fs::read_to_string(&p)
            .ok()
            .and_then(|t: String| RunSettings::parse(p, &t)),
        None => info.runsettings.clone(),
    };
    if let Some(rs) = parsed {
        println!("settings  : {}", runsettings_line(&rs));
    }
}
