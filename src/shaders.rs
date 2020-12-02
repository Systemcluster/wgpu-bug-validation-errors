use std::{
    cell::RefCell,
    path::{Path, PathBuf},
};

use shaderc::{
    CompilationArtifact, CompileOptions, Compiler, Error, OptimizationLevel, ResolvedInclude,
    ShaderKind, SourceLanguage,
};


fn get_compiler() -> Option<Compiler> { Compiler::new() }

fn get_compile_options<'a>(shader_path: &str) -> Option<CompileOptions<'a>> {
    let mut options = CompileOptions::new()?;
    options.set_source_language(SourceLanguage::HLSL);
    if cfg!(debug_assertions) {
        options.set_optimization_level(OptimizationLevel::Performance);
        options.set_generate_debug_info();
    } else {
        options.set_optimization_level(OptimizationLevel::Performance);
    }
    options.set_auto_bind_uniforms(true);
    let base = std::env::current_dir().unwrap();
    let shader_path = base.join(shader_path);
    options.set_include_callback(move |_file, _include_type, _source, _depth| {
        let file_path = shader_path.join(_file);
        Ok(ResolvedInclude {
            resolved_name: file_path.to_str().unwrap().to_owned(),
            content:       std::fs::read_to_string(file_path).map_err(|e| e.to_string())?,
        })
    });
    Some(options)
}

fn compile_shader<P: AsRef<Path>>(
    file: &P, compiler: &mut Compiler, options: &CompileOptions,
) -> Result<CompilationArtifact, Error> {
    let source = std::fs::read_to_string(&file).map_err(|e| Error::InternalError(e.to_string()))?;
    let shader = compiler.compile_into_spirv(
        source.as_str(),
        ShaderKind::InferFromSource,
        file.as_ref().file_name().unwrap().to_str().unwrap(),
        "main",
        Some(&options),
    )?;
    Ok(shader)
}

thread_local! {
    static SHADER_COMPILER: RefCell<Compiler> =
        RefCell::new(get_compiler().expect("couldn't create shader compiler"));
    static SHADER_COMPILER_OPTIONS: CompileOptions<'static> =
        get_compile_options("data/shaders").expect("couldn't create shader options");
}

pub fn get_shader<P: AsRef<Path>>(path: P) -> Result<Vec<u32>, Error> {
    SHADER_COMPILER_OPTIONS.with(|options| {
        SHADER_COMPILER.with(|compiler| {
            let path = std::env::current_dir()
                .unwrap()
                .join("data/shaders")
                .join(path.as_ref());
            let mut path = path.as_os_str().to_owned();
            path.push(".hlsl");
            let path = PathBuf::from(path);
            let shader = compile_shader(&path, &mut compiler.borrow_mut(), &options)
                .map(|artifact| artifact.as_binary().to_owned());
            shader
        })
    })
}
