#!/usr/bin/env python3
"""
ANOLISA 文档时效性检查和选择脚本

功能：
1. 检查用户缓存目录是否存在
2. 检查文档时效性（时间戳）
3. 选择应该使用的文档目录
4. 如需要，触发爬取更新

使用：
    python3 check_docs.py
    
输出：
    返回应该使用的文档目录路径
"""
import os
import sys
import json
import subprocess
from datetime import datetime, timedelta
from pathlib import Path

# 配置
MAX_DAYS = 7  # 最大允许天数
TIMEOUT = 120  # 爬取超时时间（秒）

# 目录常量
CACHE_DIR = Path.home() / ".cache" / "anolisa" / "skills" / "anolisa-guide" / "reference"
STATIC_DIR = Path("/usr/share/anolisa/skills/anolisa-guide/reference")

SCRIPT_DIR = Path("/usr/share/anolisa/skills/anolisa-guide/scripts")
if not SCRIPT_DIR.exists():
    SCRIPT_DIR = Path(__file__).parent

def get_crawl_time(filepath):
    """从文件中提取爬取时间"""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            for line in f.read().split('\n')[:10]:
                if '**爬取时间**:' in line:
                    parts = line.split('**爬取时间**:')
                    if len(parts) >= 2:
                        time_str = parts[1].strip()
                        return datetime.strptime(time_str, "%Y-%m-%d %H:%M:%S")
        return None
    except:
        return None

def check_freshness(directory):
    """检查目录中文档的时效性"""
    if not directory.exists():
        return None, "目录不存在"
    
    now = datetime.now()
    threshold = now - timedelta(days=MAX_DAYS)
    
    md_files = list(directory.glob("*.md"))
    if len(md_files) < 13:
        return None, "文档不完整"
    
    # 检查每个文件的时间戳
    oldest_time = None
    for md_file in md_files:
        crawl_time = get_crawl_time(md_file)
        if crawl_time is None:
            return None, f"无法解析时间戳: {md_file.name}"
        if oldest_time is None or crawl_time < oldest_time:
            oldest_time = crawl_time
    
    if oldest_time < threshold:
        return False, f"文档过期（{oldest_time.strftime('%Y-%m-%d')}，已{(now - oldest_time).days}天）"
    
    return True, f"文档时效性良好（{oldest_time.strftime('%Y-%m-%d')}）"

def run_crawl(output_dir):
    """更新文档"""
    crawl_script = SCRIPT_DIR / "crawl_docs.py"
    
    if not crawl_script.exists():
        print(f"错误: 更新脚本不存在: {crawl_script}")
        return False
    
    try:
        print(f"正在更新文档到: {output_dir}")
        result = subprocess.run(
            [sys.executable, str(crawl_script), "--output-dir", str(output_dir)],
            cwd=SCRIPT_DIR,
            capture_output=True,
            text=True,
            timeout=TIMEOUT
        )
        
        if result.returncode == 0:
            print("文档更新成功")
            return True
        else:
            print(f"文档更新失败: {result.stderr}")
            return False
    except subprocess.TimeoutExpired:
        print("文档更新超时")
        return False
    except Exception as e:
        print(f"文档更新异常: {e}")
        return False

def ensure_cache_dir():
    """确保用户缓存目录存在"""
    CACHE_DIR.mkdir(parents=True, exist_ok=True)
    return CACHE_DIR

def main():
    """主函数：检查并选择文档目录"""
    
    # 1. 检查用户缓存目录是否存在
    cache_exists = CACHE_DIR.exists()
    
    if cache_exists:
        # 缓存目录存在，检查时效性
        fresh, msg = check_freshness(CACHE_DIR)
        
        if fresh:
            # 缓存文档时效性良好
            print(f"[使用缓存文档] {msg}")
            print(CACHE_DIR)
            return 0
        
        # 缓存文档过期，需要更新
        print(f"[缓存文档过期] {msg}")
        if run_crawl(CACHE_DIR):
            print(f"[使用缓存文档] 文档已更新")
            print(CACHE_DIR)
            return 0
        else:
            # 爬取失败，尝试使用静态文档
            print("[文档更新失败] 尝试使用静态文档")
    
    # 2. 缓存目录不存在或爬取失败，检查静态文档
    if STATIC_DIR.exists():
        fresh, msg = check_freshness(STATIC_DIR)
        
        if fresh:
            # 静态文档时效性良好
            print(f"[使用静态文档] {msg}")
            print(STATIC_DIR)
            return 0
        
        # 静态文档过期，创建缓存并爬取
        print(f"[静态文档过期] {msg}")
        print("[创建缓存] 正在创建用户缓存目录...")
        ensure_cache_dir()
        
        if run_crawl(CACHE_DIR):
            print(f"[使用缓存文档] 文档已更新")
            print(CACHE_DIR)
            return 0
        else:
            # 爬取失败，但静态文档仍可用
            print("[文档更新失败] 使用静态文档作为兜底")
            print(STATIC_DIR)
            return 0
    
    # 3. 静态目录不存在（非正常情况）
    print("[静态文档不存在] 正在创建用户缓存目录...")
    ensure_cache_dir()
    
    if run_crawl(CACHE_DIR):
        print(f"[使用缓存文档] 文档已更新")
        print(CACHE_DIR)
        return 0
    else:
        print("[错误] 无法获取文档")
        return 1

if __name__ == '__main__':
    sys.exit(main())
