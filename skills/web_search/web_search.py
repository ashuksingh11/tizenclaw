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
    import os, json
    claw_args = os.environ.get("CLAW_ARGS")
    if claw_args:
        try:
            parsed = json.loads(claw_args)
            for k, v in parsed.items():
                globals()[k] = v # crude but effective mapping for args
            
            # Simple wrapper mapping based on script name
            script_name = os.path.basename(__file__)
            if "launch_app" in script_name:
                launch_app(parsed.get("app_id", ""))
                sys.exit(0)
            elif "vibrate_device" in script_name:
                print(json.dumps(vibrate(parsed.get("duration_ms", 1000))))
                sys.exit(0)
            elif "schedule_alarm" in script_name:
                print(json.dumps(schedule_prompt(parsed.get("delay_sec", 600), parsed.get("prompt_text", ""))))
                sys.exit(0)
            elif "web_search" in script_name:
                print(json.dumps(search_wikipedia(parsed.get("query", ""))))
                sys.exit(0)
        except Exception as e:
            print(json.dumps({"error": f"Failed to parse CLAW_ARGS: {e}"}))

    if len(sys.argv) < 2:
        print(json.dumps({"error": "No query provided"}))
        sys.exit(1)
        
    query = " ".join(sys.argv[1:])
    result = search_wikipedia(query)
    print(json.dumps(result))
