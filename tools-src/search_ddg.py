import sys
import json
import time
from typing import List, Dict

def search_ddg(query: str, max_results: int = 5) -> List[Dict[str, str]]:
    """
    Sovereign search via DuckDuckGo.
    Resilient against Rate Limits via backoff.
    """
    try:
        try:
            from ddgs import DDGS
        except ImportError:
            try:
                from duckduckgo_search import DDGS
            except ImportError:
                return [{"error": "DuckDuckGo search package not installed. Please run 'pip install duckduckgo-search'."}]
    except Exception as e:
        return [{"error": f"Import error: {str(e)}"}]
    
    max_retries = 3
    base_delay = 2
    
    for attempt in range(max_retries):
        try:
            with DDGS() as ddgs:
                results = list(ddgs.text(query, max_results=max_results))
                if not results:
                    return []
                
                return [
                    {
                        "title": r.get('title', ''),
                        "url": r.get('href', ''),
                        "body": r.get('body', '')
                    }
                    for r in results
                ]
                
        except Exception as e:
            err_str = str(e).lower()
            is_rate_limit = any(x in err_str for x in ["29", "429", "rate limit", "too many requests"])
            
            if is_rate_limit and attempt < max_retries - 1:
                wait_time = base_delay * (2 ** attempt)
                time.sleep(wait_time)
                continue
            
            return [{"error": f"Search failed: {str(e)}"}]
    
    return [{"error": "Search failed: Max retries exceeded (Rate Limit)."}]

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(json.dumps([{"error": "No query provided"}]))
        sys.exit(1)
    
    query = sys.argv[1]
    max_results = 5
    if len(sys.argv) > 2:
        try:
            max_results = int(sys.argv[2])
        except ValueError:
            pass
            
    results = search_ddg(query, max_results)
    print(json.dumps(results))
