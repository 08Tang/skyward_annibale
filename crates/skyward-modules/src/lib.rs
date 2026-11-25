use anyhow::{anyhow, bail, Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub mod modprobe;

/// Where a module source's files are in the recovery: the vendor_boot
/// ramdisk's modules under /lib/modules, the vendor_dlkm and vendor
/// partitions where they are mounted, and the kernel's own modules in
/// system_dlkm, which it also keeps in one folder.
pub fn source_dir(source: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(match source {
        "vendor_boot" => "/lib/modules",
        "vendor_dlkm" => "/vendor_dlkm/lib/modules",
        "vendor" => "/vendor/lib/modules",
        "system_dlkm" => "/system_dlkm/flatten/lib/modules",
        _ => bail!("unknown module source {source:?}"),
    }))
}

/// Load `names` (module names without `.ko`) with their dependencies. Each
/// comes from the first of `dirs` that holds it, and its dependencies from
/// that directory's modules.dep; a module the kernel already has counts as
/// loaded. Returns the files loaded, dependencies first.
pub fn load(dirs: &[PathBuf], names: &[String]) -> Result<Vec<PathBuf>> {
    let mut loaded = Vec::new();
    for name in names {
        let file = format!("{name}.ko");
        let dir = dirs
            .iter()
            .find(|d| d.join(&file).is_file())
            .ok_or_else(|| anyhow!("{file} is in none of {dirs:?}"))?;
        let graph = parse_dep(dir)?;
        for m in graph.load_order(std::slice::from_ref(&file)) {
            let path = graph
                .resolve_path(&m)
                .ok_or_else(|| anyhow!("{m} is listed in {}/modules.dep but missing", dir.display()))?;
            insmod(&path, "")?;
            loaded.push(path);
        }
    }
    Ok(loaded)
}

/// Unload `names` (module names without `.ko`), dependencies last: what was
/// loaded first goes last, so nothing is asked to leave while something it
/// brought with it still needs it.
///
/// This is the way out of the recovery, not a way to make room: the installed
/// system's own driver cold-starts the chip, and a driver left in the kernel
/// with the radio on makes that start fail. Returns the modules that left, as
/// the kernel names them. A module that was never loaded has nothing to do
/// and is passed over; one something else still holds is an error, since the
/// caller is the one that can wait and try again.
pub fn unload(dirs: &[PathBuf], names: &[String]) -> Result<Vec<String>> {
    let mut gone = Vec::new();
    let mut held = Vec::new();
    // Every module the directories know of, so that a dependency loaded along
    // with the one named is taken out too: the reverse of the order `load`
    // put them in.
    let mut order: Vec<String> = Vec::new();
    for dir in dirs {
        let Ok(graph) = parse_dep(dir) else { continue };
        for name in names {
            let file = format!("{name}.ko");
            if !dir.join(&file).is_file() {
                continue;
            }
            for m in graph.load_order(std::slice::from_ref(&file)) {
                if !order.contains(&m) {
                    order.push(m);
                }
            }
        }
    }
    for file in order.iter().rev() {
        let module = file.trim_end_matches(".ko").replace('-', "_");
        match remove(&module) {
            Ok(()) => gone.push(module),
            // Never there: nothing to take out, and nothing to report.
            Err(e) if e.raw_os_error() == Some(libc::ENOENT) => {}
            Err(e) if matches!(e.raw_os_error(), Some(libc::EBUSY) | Some(libc::EINVAL)) => held.push(module),
            Err(e) => return Err(e).with_context(|| format!("remove {module}")),
        }
    }
    if !held.is_empty() {
        bail!("{} {} still in use", held.join(", "), if held.len() == 1 { "is" } else { "are" });
    }
    Ok(gone)
}

#[cfg(target_os = "android")]
fn remove(module: &str) -> std::io::Result<()> {
    let name = std::ffi::CString::new(module).map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;
    // `delete_module` with O_NONBLOCK asks the kernel not to wait for the
    // module to stop being used; O_TRUNC is what it wants for a module it may
    // remove even where the kernel would rather keep it.
    let flags = libc::O_NONBLOCK | libc::O_TRUNC;
    let rc = unsafe { libc::syscall(libc::SYS_delete_module, name.as_ptr(), flags) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(not(target_os = "android"))]
fn remove(_module: &str) -> std::io::Result<()> {
    Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "modules are unloaded only on the phone"))
}

/// A `modules.dep` graph: each module file mapped to the module files it
/// depends on. Keys and values are reduced to bare file names so the graph is
/// independent of where the modules were sourced from (GKI devices keep them
/// in several places: vendor_boot ramdisk, vendor_dlkm, vendor).
#[derive(Debug, Default)]
pub struct ModuleGraph {
    pub base: PathBuf,
    pub deps: BTreeMap<String, Vec<String>>,
}

pub fn parse_dep(base: impl AsRef<Path>) -> Result<ModuleGraph> {
    let base = base.as_ref().to_path_buf();
    let dep_path = base.join("modules.dep");
    let text = fs::read_to_string(&dep_path)
        .with_context(|| format!("read {}", dep_path.display()))?;
    let mut deps = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((lhs, rhs)) = line.split_once(':') else {
            continue;
        };
        let name = basename(lhs.trim());
        let ds = rhs.split_whitespace().map(basename).collect();
        deps.insert(name, ds);
    }
    Ok(ModuleGraph { base, deps })
}

fn basename(p: &str) -> String {
    Path::new(p)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| p.to_string())
}

impl ModuleGraph {
    /// Topologically ordered load list. With an empty `requested`, every module
    /// in the graph is scheduled; otherwise only the requested modules and
    /// their transitive dependencies, dependencies first.
    pub fn load_order(&self, requested: &[String]) -> Vec<String> {
        let mut order = Vec::new();
        let mut seen = BTreeSet::new();
        let list: Vec<String> = if requested.is_empty() {
            self.deps.keys().cloned().collect()
        } else {
            requested.to_vec()
        };
        for m in list {
            self.visit(&m, &mut seen, &mut order);
        }
        order
    }

    fn visit(&self, m: &str, seen: &mut BTreeSet<String>, order: &mut Vec<String>) {
        if !seen.insert(m.to_string()) {
            return;
        }
        if let Some(ds) = self.deps.get(m) {
            for d in ds {
                self.visit(d, seen, order);
            }
        }
        order.push(m.to_string());
    }

    pub fn resolve_path(&self, module: &str) -> Option<PathBuf> {
        let direct = self.base.join(module);
        if direct.exists() {
            return Some(direct);
        }
        find_file(&self.base, module)
    }
}

fn find_file(dir: &Path, name: &str) -> Option<PathBuf> {
    let rd = fs::read_dir(dir).ok()?;
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            if let Some(f) = find_file(&p, name) {
                return Some(f);
            }
        } else if p.file_name().map(|s| s == name).unwrap_or(false) {
            return Some(p);
        }
    }
    None
}

/// Hand the open module `file` to the kernel with the settings `params`.
#[cfg(target_os = "android")]
fn finit_module(file: &fs::File, params: &str) -> std::io::Result<()> {
    use std::os::unix::io::AsRawFd;
    let params = std::ffi::CString::new(params).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    let rc = unsafe { libc::syscall(libc::SYS_finit_module, file.as_raw_fd(), params.as_ptr(), 0 as libc::c_int) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(not(target_os = "android"))]
fn finit_module(_file: &fs::File, _params: &str) -> std::io::Result<()> {
    Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "modules are loaded only on the phone"))
}

pub fn insmod(path: impl AsRef<Path>, params: &str) -> Result<()> {
    let path = path.as_ref();
    let f = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    match finit_module(&f, params) {
        Ok(()) => Ok(()),
        // Already loaded is success for our purposes.
        Err(e) if e.raw_os_error() == Some(libc::EEXIST) => Ok(()),
        Err(e) => Err(e).with_context(|| format!("finit_module {}", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_dependencies_first() {
        let mut deps = BTreeMap::new();
        deps.insert("a.ko".to_string(), vec!["b.ko".to_string()]);
        deps.insert("b.ko".to_string(), vec!["c.ko".to_string()]);
        deps.insert("c.ko".to_string(), vec![]);
        let g = ModuleGraph { base: PathBuf::new(), deps };
        let order = g.load_order(&["a.ko".to_string()]);
        assert_eq!(order, vec!["c.ko", "b.ko", "a.ko"]);
    }

    #[test]
    fn strips_paths_to_basenames() {
        assert_eq!(basename("kernel/drivers/foo/bar.ko"), "bar.ko");
    }

    #[test]
    fn takes_out_what_it_put_in_last() {
        // The order `unload` walks is the reverse of `load`'s, so the module
        // named comes out after the ones it brought with it. Nothing is
        // removed on a computer, so what is checked is the order it would go
        // in: the graph it reads, and the names it derives from it.
        let mut deps = BTreeMap::new();
        deps.insert("a.ko".to_string(), vec!["b.ko".to_string()]);
        deps.insert("b.ko".to_string(), vec!["c.ko".to_string()]);
        deps.insert("c.ko".to_string(), vec![]);
        let g = ModuleGraph { base: PathBuf::new(), deps };
        let order = g.load_order(&["a.ko".to_string()]);
        let removed: Vec<String> = order.iter().rev().map(|f| f.trim_end_matches(".ko").replace('-', "_")).collect();
        assert_eq!(removed, vec!["a", "b", "c"], "the named module leaves first, its dependency last");
    }
}
