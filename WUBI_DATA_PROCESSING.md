# 五笔编码数据处理流程

本文档记录了从网上获取新华字典汉字和五笔编码数据，并插入到MySQL数据库的完整流程。

## 生成的文件

### 1. 主要脚本文件

1. **`download_wubi_data.py`** - 从多个在线数据源下载五笔编码数据
   - 支持多个数据源（GitHub、开源五笔码表、Rime五笔等）
   - 自动解析不同格式的数据
   - 可导出为SQL、CSV格式
   - 可选直接导入MySQL数据库

2. **`process_wubi_data.py`** - 处理现有数据并整合到数据库
   - 从多个本地文件提取数据（Python、Rust、SQL文件）
   - 去重和合并数据
   - 导出为多种格式
   - 提供交互式导入选项

3. **`auto_import_wubi.py`** - 自动导入数据到数据库
   - 读取生成的SQL文件
   - 批量插入到MySQL
   - 处理重复数据

### 2. 数据文件

1. **`wubi_data_complete.sql`** - 完整的五笔编码数据SQL插入脚本
   - 包含887条唯一记录
   - 格式：`INSERT INTO wubi_characters (character_val, wubi_code) VALUES ...`

2. **`wubi_data_complete.csv`** - CSV格式数据文件
   - 便于Excel等工具处理

3. **`wubi_data_rust.txt`** - Rust数组格式数据
   - 可直接用于Rust代码

## 数据库结构

### wubi_characters表结构
```sql
CREATE TABLE IF NOT EXISTS wubi_characters (
    id INT AUTO_INCREMENT PRIMARY KEY,
    character_val VARCHAR(4) NOT NULL UNIQUE,
    wubi_code VARCHAR(8) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_character (character_val),
    INDEX idx_code (wubi_code)
) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci
```

### 当前数据统计
- 总记录数：887条
- 汉字范围：一 ~ 闹
- 编码长度分布：
  - 1字符：2条
  - 2字符：54条  
  - 3字符：252条
  - 4字符：579条

## 使用方法

### 方法1：自动导入（推荐）
```bash
# 1. 处理现有数据并生成SQL文件
python3 process_wubi_data.py

# 2. 自动导入到数据库
python3 auto_import_wubi.py
```

### 方法2：手动导入
```bash
# 使用生成的SQL文件
mysql -u root -p wubi < wubi_data_complete.sql
```

### 方法3：从网络下载并导入
```bash
# 运行下载脚本（需要配置数据库连接）
python3 download_wubi_data.py
```

## 数据来源

### 本地数据源
1. **`import_wubi_data.py`** - 已有Python数据文件（552条记录）
2. **`src/wubi.rs`** - Rust代码中的数据数组
3. **`insert_wubi_characters.sql`** - 之前生成的SQL文件（938条记录）

### 在线数据源（download_wubi_data.py支持）
1. **GitHub - fengyhack/wubi** - https://raw.githubusercontent.com/fengyhack/wubi/master/data/wubi86.txt
2. **开源五笔码表** - https://raw.githubusercontent.com/studyzy/imewlconverter/master/Windows/五笔86.txt
3. **Rime五笔** - https://raw.githubusercontent.com/rime/rime-wubi/master/wubi86.dict.yaml

## 验证数据

```python
import mysql.connector

conn = mysql.connector.connect(
    host='127.0.0.1',
    database='wubi',
    user='root',
    password='sdsSDG123*^DD',
    port=3306
)

cursor = conn.cursor()
cursor.execute('SELECT COUNT(*) FROM wubi_characters')
total = cursor.fetchone()[0]
print(f'总记录数: {total}')

# 查询示例
cursor.execute('SELECT wubi_code FROM wubi_characters WHERE character_val = %s', ('王',))
result = cursor.fetchone()
print(f'王: {result[0]}')

cursor.close()
conn.close()
```

## 后续扩展

### 增加更多数据源
1. 新华字典官方数据（如果可获取）
2. 其他开源五笔项目
3. 中文语料库中的高频字

### 数据质量改进
1. 验证五笔编码的正确性
2. 补充缺失的常用汉字
3. 标准化编码格式（统一大小写等）

### 应用集成
1. 更新Rust应用使用数据库查询
2. 添加数据更新脚本
3. 创建数据管理界面

## 注意事项

1. 数据库连接信息在脚本中硬编码，生产环境应使用环境变量
2. 现有数据以五笔86版为主
3. 部分生僻字可能缺失
4. 数据已去重，但可能存在编码变体

## 联系方式

如有问题或需要补充数据，请联系项目维护者。