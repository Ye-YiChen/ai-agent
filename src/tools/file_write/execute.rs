use std::path::Path;

use super::r#impl::WriteFileArgs;

/// 实际的文件写入逻辑：按需创建父目录，支持覆盖/追加，返回写入的字节数。
pub async fn write(args: &WriteFileArgs) -> anyhow::Result<usize> {
    let path = Path::new(&args.file_path);

    // 确保父目录存在
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }

    let bytes = args.content.len();

    if args.append {
        use tokio::io::AsyncWriteExt;
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&args.file_path)
            .await?;
        file.write_all(args.content.as_bytes()).await?;
    } else {
        tokio::fs::write(&args.file_path, args.content.as_bytes()).await?;
    }

    Ok(bytes)
}
