use std::path::Path;
use std::fs;
use std::io;
use std::process::Command;

pub fn extract_zip(zip_path: &Path, extract_to: &Path) -> Result<(), String> {
    let file = fs::File::open(zip_path)
        .map_err(|e| format!("Failed to open zip file: {}", e))?;
    
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Failed to read zip archive: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read file from archive: {}", e))?;
        
        let outpath = match file.enclosed_name() {
            Some(path) => extract_to.join(path),
            None => continue,
        };

        if file.is_dir() {
            fs::create_dir_all(&outpath)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)
                        .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                }
            }
            let mut outfile = fs::File::create(&outpath)
                .map_err(|e| format!("Failed to create file: {}", e))?;
            io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to extract file: {}", e))?;
        }
    }

    Ok(())
}

pub fn create_backup(source_path: &Path, backup_dir: &Path) -> Result<String, String> {
    if !source_path.exists() {
        return Err("Source path does not exist".to_string());
    }

    fs::create_dir_all(backup_dir)
        .map_err(|e| format!("Failed to create backup directory: {}", e))?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!(
        "{}_{}", 
        source_path.file_name().unwrap_or_default().to_string_lossy(),
        timestamp
    );
    let backup_path = backup_dir.join(&backup_name);

    if source_path.is_dir() {
        copy_dir_recursive(source_path, &backup_path)?;
    } else {
        fs::copy(source_path, &backup_path)
            .map_err(|e| format!("Failed to backup file: {}", e))?;
    }

    Ok(backup_path.to_string_lossy().to_string())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    for entry in fs::read_dir(src)
        .map_err(|e| format!("Failed to read directory: {}", e))? 
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let file_type = entry.file_type()
            .map_err(|e| format!("Failed to get file type: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("Failed to copy file: {}", e))?;
        }
    }

    Ok(())
}

pub fn run_post_install_script(
    script_path: &Path,
    working_dir: &Path,
) -> Result<(), String> {
    // Verify script exists
    if !script_path.exists() {
        return Err(format!("Script not found: {}", script_path.display()));
    }

    // Security check: ensure script is within the working directory
    let script_canonical = script_path.canonicalize()
        .map_err(|e| format!("Failed to resolve script path: {}", e))?;
    let working_canonical = working_dir.canonicalize()
        .map_err(|e| format!("Failed to resolve working directory: {}", e))?;
    
    if !script_canonical.starts_with(&working_canonical) {
        return Err("Security error: Script must be inside game directory".to_string());
    }

    println!("Running post-install script: {}", script_path.display());

    // Run the batch file on Windows
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("cmd")
            .args(["/C", &script_path.to_string_lossy()])
            .current_dir(working_dir)
            .output()
            .map_err(|e| format!("Failed to execute script: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !stdout.is_empty() {
            println!("Script output:\n{}", stdout);
        }
        if !stderr.is_empty() {
            eprintln!("Script errors:\n{}", stderr);
        }

        if !output.status.success() {
            let exit_code = output.status.code().unwrap_or(-1);
            eprintln!("Warning: Script exited with code {}", exit_code);
            // Don't return error - just log the warning as per requirements
        } else {
            println!("Script completed successfully");
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        eprintln!("Warning: Post-install scripts only supported on Windows");
    }

    Ok(())
}








