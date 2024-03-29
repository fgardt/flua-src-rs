use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Build {
    out_dir: Option<PathBuf>,
    target: Option<String>,
    host: Option<String>,
}

pub struct Artifacts {
    include_dir: PathBuf,
    lib_dir: PathBuf,
    libs: Vec<String>,
    cpp_stdlib: Option<String>,
}

impl Build {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Build {
        Build {
            out_dir: env::var_os("OUT_DIR").map(|s| PathBuf::from(s).join("lua-build")),
            target: env::var("TARGET").ok(),
            host: env::var("HOST").ok(),
        }
    }

    pub fn out_dir<P: AsRef<Path>>(&mut self, path: P) -> &mut Build {
        self.out_dir = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn target(&mut self, target: &str) -> &mut Build {
        self.target = Some(target.to_string());
        self
    }

    pub fn host(&mut self, host: &str) -> &mut Build {
        self.host = Some(host.to_string());
        self
    }

    pub fn build(&self) -> Artifacts {
        const LIB_NAME: &str = "flua";
        let target = &self.target.as_ref().expect("TARGET not set")[..];
        let host = &self.host.as_ref().expect("HOST not set")[..];
        let out_dir = self.out_dir.as_ref().expect("OUT_DIR not set");
        let lib_dir = out_dir.join("lib");
        let include_dir = out_dir.join("include");
        let source_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("flua/src");

        if lib_dir.exists() {
            fs::remove_dir_all(&lib_dir).unwrap();
        }
        fs::create_dir_all(&lib_dir).unwrap();

        if include_dir.exists() {
            fs::remove_dir_all(&include_dir).unwrap();
        }
        fs::create_dir_all(&include_dir).unwrap();

        let mut config = cc::Build::new();
        config
            .target(target)
            .host(host)
            .warnings(false)
            .opt_level(2)
            .cargo_metadata(false)
            .define("LUA_COMPAT_ALL", None)
            .define("LUA_USE_STRTODHEX", None)
            .define("LUA_USE_AFORMAT", None)
            .define("LUA_USE_LONGLONG", None)
            .define("USE_LUA_PACKAGE", None)
            .define("USE_LUA_COROUTINE", None)
            .flag_if_supported("-std=c++14")
            .flag_if_supported("/std:c++14") // MSVC
            .cpp(true);

        match target {
            _ if target.contains("linux") => {
                config
                    .define("LUA_USE_LINUX", None)
                    .define("LUA_USE_POSIX", None);
            }
            _ if target.contains("freebsd") => {
                config
                    .define("LUA_USE_LINUX", None)
                    .define("LUA_USE_POSIX", None);
            }
            _ if target.contains("netbsd") => {
                config
                    .define("LUA_USE_LINUX", None)
                    .define("LUA_USE_POSIX", None);
            }
            _ if target.contains("openbsd") => {
                config
                    .define("LUA_USE_LINUX", None)
                    .define("LUA_USE_POSIX", None);
            }
            _ if target.contains("apple-darwin") => {
                config
                    .define("LUA_USE_MACOSX", None)
                    .define("LUA_USE_POSIX", None);
            }
            _ if target.contains("windows") => {}
            _ => panic!("don't know how to build Factorio Lua for {}", target),
        }

        if cfg!(debug_assertions) {
            config.define("LUA_USE_APICHECK", None);
        }

        config
            .include(&source_dir)
            .flag("-w")
            .flag_if_supported("-fno-common")
            .file(source_dir.join("lapi.c"))
            .file(source_dir.join("lauxlib.c"))
            .file(source_dir.join("lbaselib.c"))
            .file(source_dir.join("lbitlib.c"))
            .file(source_dir.join("lcode.c"))
            .file(source_dir.join("lcorolib.c"))
            .file(source_dir.join("lctype.c"))
            .file(source_dir.join("ldblib.c"))
            .file(source_dir.join("ldebug.c"))
            .file(source_dir.join("ldo.c"))
            .file(source_dir.join("ldump.c"))
            .file(source_dir.join("lfunc.c"))
            .file(source_dir.join("lgc.c"))
            .file(source_dir.join("linit.c"))
            .file(source_dir.join("liolib.c"))
            .file(source_dir.join("llex.c"))
            .file(source_dir.join("lmathlib.c"))
            .file(source_dir.join("lmem.c"))
            .file(source_dir.join("loadlib.c"))
            .file(source_dir.join("lobject.c"))
            .file(source_dir.join("lopcodes.c"))
            .file(source_dir.join("loslib.c"))
            .file(source_dir.join("lparser.c"))
            .file(source_dir.join("lstate.c"))
            .file(source_dir.join("lstring.c"))
            .file(source_dir.join("lstrlib.c"))
            .file(source_dir.join("ltable.c"))
            .file(source_dir.join("ltablib.c"))
            .file(source_dir.join("ltm.c"))
            .file(source_dir.join("lundump.c"))
            .file(source_dir.join("lvm.c"))
            .file(source_dir.join("lzio.c"))
            .out_dir(&lib_dir)
            .compile(LIB_NAME);

        for f in &["lauxlib.h", "lua.h", "luaconf.h", "lualib.h"] {
            fs::copy(source_dir.join(f), include_dir.join(f)).unwrap();
        }

        fs::copy(source_dir.join("lua.hpp"), include_dir.join("lua.hpp")).unwrap();

        Artifacts {
            lib_dir,
            include_dir,
            libs: vec![LIB_NAME.to_string()],
            cpp_stdlib: Self::get_cpp_link_stdlib(target),
        }
    }

    fn get_cpp_link_stdlib(target: &str) -> Option<String> {
        // Copied from the `cc` crate
        if target.contains("msvc") {
            None
        } else if target.contains("apple") {
            Some("c++".to_string())
        } else if target.contains("freebsd") {
            Some("c++".to_string())
        } else if target.contains("openbsd") {
            Some("c++".to_string())
        } else if target.contains("android") {
            Some("c++_shared".to_string())
        } else {
            Some("stdc++".to_string())
        }
    }
}

impl Artifacts {
    pub fn include_dir(&self) -> &Path {
        &self.include_dir
    }

    pub fn lib_dir(&self) -> &Path {
        &self.lib_dir
    }

    pub fn libs(&self) -> &[String] {
        &self.libs
    }

    pub fn print_cargo_metadata(&self) {
        println!("cargo:rustc-link-search=native={}", self.lib_dir.display());
        for lib in self.libs.iter() {
            println!("cargo:rustc-link-lib=static={}", lib);
        }
        if let Some(ref cpp_stdlib) = self.cpp_stdlib {
            println!("cargo:rustc-link-lib={}", cpp_stdlib);
        }
        println!("cargo:include={}", self.include_dir.display());
        println!("cargo:lib={}", self.lib_dir.display());
    }
}
