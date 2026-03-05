#!/usr/bin/env python3
"""
TizenClaw Skill: Web Search
Standard OpenClaw-compatible skill to search Wikipedia.
"""
import urllib.request
import urllib.parse
import json
import sys

def search_wikipedia(query):
    try:
        url = f"https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch={urllib.parse.quote(query)}&utf8=&format=json"
        req = urllib.request.Request(url, headers={'User-Agent': 'TizenClaw/1.0'})
        with urllib.request.urlopen(req, timeout=10) as response:
            data = json.loads(response.read().decode())
            results = data.get("query", {}).get("search", [])
            
            # Return top 2 results
            summaries = []
            for res in results[:2]:
                title = res.get("title", "")
                snippet = res.get("snippet", "").replace('<span class="searchmatch">', '').replace('</span>', '')
                summaries.append({"title": title, "snippet": snippet})
                
            return {"results": summaries}
            
    except Exception as e:
        return {"error": str(e)}

if __name__ == "__main__":
    import os
    claw_args = os.environ.get("CLAW_ARGS")
    if claw_args:
        try:
            parsed = json.loads(claw_args)
            query = parsed.get("query", "")
            if query:
                print(json.dumps(search_wikipedia(query)))
                sys.exit(0)
        except Exception as e:
            print(json.dumps({"error": f"Failed to parse CLAW_ARGS: {e}"}))
            sys.exit(1)

    if len(sys.argv) < 2:
        print(json.dumps({"error": "No query provided"}))
        sys.exit(1)
        
    query = " ".join(sys.argv[1:])
    print(json.dumps(search_wikipedia(query)))
