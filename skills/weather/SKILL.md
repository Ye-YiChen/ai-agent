---
name: weather
description: 查询任意城市的当前天气和预报，无需 API 密钥。当用户问天气、气温、下不下雨、未来几天天气时使用。
---

# 天气查询技能

用免费公共服务查天气，无需 API 密钥。通过 run_script 工具执行下面的命令。

## 首选：wttr.in（返回可读文本）
- 单行简报（推荐先用这个）：
  `curl -s "wttr.in/Beijing?format=3"`
  输出示例：`Beijing: ⛅️ +8°C`
- 紧凑一行（含湿度风速）：
  `curl -s "wttr.in/Beijing?format=%l:+%c+%t+%h+%w"`
- 完整多日预报：
  `curl -s "wttr.in/Beijing?T"`
- 只看今天：末尾加 `?1`；只看当前：`?0`；公制单位加 `?m`。

## 用法要点
- 城市名里的空格用 `+` 代替：`wttr.in/New+York`。
- 也支持机场代码：`wttr.in/PEK`。
- 中文城市建议转成英文/拼音，如 北京→Beijing、上海→Shanghai。

## 备用：Open-Meteo（返回 JSON，适合程序化处理）
先拿到城市经纬度，再查：
`curl -s "https://api.open-meteo.com/v1/forecast?latitude=39.9&longitude=116.4&current_weather=true"`
返回 JSON 含温度、风速、天气代码。

## 规则
- 优先用 wttr.in 的 `format=3` 拿到简洁结果，再用自然语言转述给用户。
- 若某服务失败（网络/超时），换另一个服务重试。
- 不要编造天气数据，一切以命令返回结果为准。
