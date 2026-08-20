---
name: skill-installer
description: 从网络下载并安装一个新技能(Skill)到本地 skills 目录，然后立即使用它。当用户要求"安装/下载一个新技能""从 skillhub 装一个 skill""帮我加一个 xx 技能"并给出链接时使用。
---

# 技能安装向导

本技能教你把一个外部 Skill 下载到本地 `skills/<name>/` 并立即启用。
可用工具：run_script（执行 shell，如 curl）、download_file（下载直链到指定路径）、read_file（读取内容）。

## 判断链接来源
先看用户给的链接属于哪一类：

### A. skillhub 链接（形如 https://skillhub.cn/skills/<namespace>/<slug>）
skillhub 的文件存在腾讯云 COS，需通过它的 API 下载。一个 skill 可能包含多个文件
（如 SKILL.md、README.md、references/*.md、scripts/* 等），务必全部下载。

1. **解析出 namespace 和 slug**：URL 路径 `/skills/<namespace>/<slug>`。
   例：`https://skillhub.cn/skills/user_741dc82b/anti-fraud` → namespace=`user_741dc82b`，slug=`anti-fraud`。

2. **取版本号 version**：用 run_script 执行
   `curl -s "https://api.skillhub.cn/api/v1/skills/<slug>?namespace=<namespace>"`
   从返回 JSON 里取 **`latestVersion.version`** 字段（例如 `1.0.10`，不同 skill 值不同，不要写死）。

3. **列出文件清单**：用 run_script 执行
   `curl -s "https://api.skillhub.cn/api/v1/skills/<slug>/files?version=<version>&namespace=<namespace>"`
   返回结构为 `{"count":N,"files":[{"path":"SKILL.md","size":..,"sha256":".."}, ...]}`。
   收集所有文件的 `path`。

4. **逐个下载文件**：对清单里每个 `path`，用 download_file 下载：
   - url = `https://api.skillhub.cn/api/v1/skills/<slug>/file?path=<path>&version=<version>&namespace=<namespace>`
     （该接口会 302 重定向到 COS，download_file 会自动跟随拿到最终内容）
   - dest_path = `skills/<slug>/<path>`（例如 `skills/anti-fraud/SKILL.md`、
     `skills/anti-fraud/references/golden-cases.md`；子目录会自动创建）
   - 至少要下载到 `SKILL.md`；其余 references/scripts 等按清单一并下载，SOP 才完整。

### B. 非 skillhub 链接
- **必须是可直接下载的文件直链**（以文件结尾或明确是下载地址，如 .md/.zip）。
- 如果用户给的不是直链（而是网页/项目主页等无法直接下载的地址），**立即停止**，明确告诉用户：
  "该链接不是文件下载直链，请提供 SKILL.md 或 zip 包的直接下载地址后再试。" 不要猜测、不要继续。
- 若是直链：用 download_file 下载到 `skills/<name>/`（name 由用户指定或从文件名推断）。

## 下载之后（A/B 通用）
1. 若下载的是 **zip 包**：用 run_script 执行 `unzip -o skills/<name>/xxx.zip -d skills/<name>/` 解压。
2. 用 **read_file** 读取 `skills/<name>/SKILL.md`。
   —— 读取动作会把该技能的完整操作指南(SOP)带入当前对话上下文，等价于加载了这个新技能。
3. 严格按读到的 SKILL.md 内容执行用户的原始需求；若其中要求运行脚本，用 run_script。

## 规则
- 每一步 run_script / download_file 都会触发用户审批，属正常流程。
- 下载目标目录统一放在项目的 `skills/` 下，用技能名作子目录。
- 安装完成后主动告诉用户："技能 <name> 已安装到 skills/<name>/，现在可以使用了。"
- 只从用户明确给出的链接下载，不要自行编造或访问其它地址。
