use std::net::IpAddr;
use std::path::Path;

/// 从给定 URL 下载文件保存到本地，返回结果描述文本。
///
/// 安全：内置 SSRF 防护——请求前解析目标 host，拦截 localhost / 环回 /
/// 私有网段 / 链路本地地址，以及额外的 9./11./21./30. 段。
pub async fn download(url_str: &str, dest_path: Option<&str>) -> anyhow::Result<String> {
    let url = reqwest::Url::parse(url_str).map_err(|e| anyhow::anyhow!("非法 URL：{e}"))?;
    if !matches!(url.scheme(), "http" | "https") {
        anyhow::bail!("仅支持 http/https，收到：{}", url.scheme());
    }

    // SSRF 防护：拦截内网/环回等目标
    ensure_public_host(&url).await?;

    // 计算保存路径：未指定则取 URL 最后一段文件名，存当前目录
    let dest = match dest_path {
        Some(p) if !p.trim().is_empty() => p.trim().to_string(),
        _ => url
            .path_segments()
            .and_then(|s| s.last())
            .filter(|s| !s.is_empty())
            .unwrap_or("download.bin")
            .to_string(),
    };

    let resp = reqwest::get(url.clone())
        .await
        .map_err(|e| anyhow::anyhow!("下载请求失败：{e}"))?;
    if !resp.status().is_success() {
        anyhow::bail!("下载失败，HTTP 状态码：{}", resp.status());
    }
    let bytes = resp.bytes().await?;

    // 确保父目录存在再写文件
    let dest_path = Path::new(&dest);
    if let Some(parent) = dest_path.parent() {
        if !parent.as_os_str().is_empty() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }
    tokio::fs::write(dest_path, &bytes).await?;

    Ok(format!(
        "已下载到 {}（{} 字节）",
        dest_path.display(),
        bytes.len()
    ))
}

/// 解析 URL 的目标 host，若解析出的任一 IP 属于内网/环回/链路本地/受限段则拒绝。
async fn ensure_public_host(url: &reqwest::Url) -> anyhow::Result<()> {
    let host = url
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("URL 缺少主机名"))?;

    if host.eq_ignore_ascii_case("localhost") {
        anyhow::bail!("出于安全，拒绝访问内网地址：{host}");
    }

    let port = url.port_or_known_default().unwrap_or(443);
    let addrs = tokio::net::lookup_host((host, port))
        .await
        .map_err(|e| anyhow::anyhow!("无法解析主机 {host}：{e}"))?;

    for addr in addrs {
        if is_blocked_ip(addr.ip()) {
            anyhow::bail!("出于安全，拒绝访问内网/受限地址：{host} -> {}", addr.ip());
        }
    }
    Ok(())
}

/// 判断 IP 是否属于应拦截的内网/受限范围。
fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()      // 10/8, 172.16/12, 192.168/16
                || v4.is_link_local()   // 169.254/16
                || v4.is_unspecified()
                || v4.is_broadcast()
                // 安全规则额外要求拦截的段（10 已被 is_private 覆盖）
                || matches!(v4.octets()[0], 9 | 11 | 21 | 30)
        }
        IpAddr::V6(v6) => v6.is_loopback() || v6.is_unspecified(),
    }
}
