---
name: akshare-stock
description: A股量化数据查询，基于 Python AkShare 库获取行情、K线、财务、板块、资金流向等。当用户问 A 股股票行情、历史K线、财务数据、板块、选股时使用。
---

# A股量化 - AkShare 数据接口

通过 run_script 工具执行 Python 脚本获取 A 股数据（数据来自 AkShare 库）。

## 前置依赖
首次使用需安装（若已安装可跳过）：
`pip install akshare pandas`

建议每次执行时，把要跑的 Python 写进一个临时文件再运行，例如：
`python3 -c "import akshare as ak; print(ak.stock_zh_a_spot_em().head())"`
或写成脚本文件后 `python3 xxx.py`。

## 常用接口速查（symbol 用 6 位代码）
1. 实时行情（全市场快照）：
   `ak.stock_zh_a_spot_em()`
2. 历史 K 线（日/周/月，qfq=前复权）：
   `ak.stock_zh_a_hist(symbol="600519", period="daily", start_date="20240101", end_date="20241231", adjust="qfq")`
3. 财务：
   `ak.stock_financial_abstract_ths(symbol="600519", indicator="按报告期")`
   `ak.stock_financial_analysis_indicator(symbol="600519")`
4. 板块/行业：
   `ak.stock_board_industry_name_em()` / `ak.stock_board_industry_cons_em(symbol="半导体")`
5. 资金流向：
   `ak.stock_individual_fund_flow(stock="600519", market="sh")`
6. 龙虎榜：`ak.stock_lhb_detail_em(date="20240930")`

## 常用代码
- 平安银行 000001、贵州茅台 600519、宁德时代 300750、比亚迪 002594、招商银行 600036

## 备用方案：Baostock（更轻量，AkShare 装不上时用）
```python
import baostock as bs
lg = bs.login()
rs = bs.query_history_k_data_plus('sh.600519',
    'date,code,open,high,low,close,volume',
    start_date='20250101', end_date='20251231')
rows = []
while rs.next():
    rows.append(rs.get_row_data())
bs.logout()
print(rows[:5])
```

## 规则
- 只输出脚本真实返回的数据；接口可能因网络或目标站变动而失败，失败就重试或换备用方案。
- 数据仅供研究，不构成投资建议——回答里可附一句风险提示。
- 查询前先确认股票代码（用常用代码表，或先用 stock_zh_a_spot_em 检索）。
