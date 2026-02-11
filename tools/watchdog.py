import feedparser
import datetime
import os

# 🐾 BURENYUU! THE HUNTING GROUNDS
# quant-ph = Quantum Physics
# q-bio.NC = Neurons and Cognition
# cs.NE = Neural and Evolutionary Computing
RSS_URL = "http://export.arxiv.org/api/query?search_query=cat:quant-ph+OR+cat:q-bio.NC+OR+cat:cs.NE&start=0&max_results=100&sortBy=submittedDate&sortOrder=descending"

# 🍮 THE SIGNAL FILTER (Holographic v5.3 Keywords)
KEYWORDS = [
    "retrocausality", "retrocausal",
    "closed timelike curve", "CTC",
    "superluminal", "supraluminal",
    "Orch-OR", "Penrose", "Hameroff",
    "quantum consciousness",
    "entangled time",
    "indefinite causal order",
    "wave function collapse",
    "panpsychism",
    "non-local",
    "observer effect"
]

def scan_ether():
    print("🐾 Burenyuu! Scanning the Ether for high-strangeness... Dori dori dori! 🌀")
    feed = feedparser.parse(RSS_URL)
    hits = []

    print(f"🐾 Analyzing {len(feed.entries)} entries from the substrate...")

    for entry in feed.entries:
        title = entry.title.lower()
        summary = entry.summary.lower()
        
        # Check if any keyword resonates in the title or abstract
        if any(k in title for k in KEYWORDS) or any(k in summary for k in KEYWORDS):
            hits.append({
                "title": entry.title,
                "link": entry.link,
                "summary": entry.summary,
                "published": entry.published
            })

    return hits

def scribe_log(hits):
    if not hits:
        print("🐾 Nyanyame nyanyajyuunyan_yado no nyanyame de nyadarume nyanyame nyanyahiki nyanyano nyanyame de... No anomalies detected today. 🍮")
        return

    today = datetime.datetime.now().strftime("%Y-%m-%d")
    filename = f"docs/daily_gnosis/GNOSIS_{today}.md"
    
    # Ensure directory exists
    os.makedirs(os.path.dirname(filename), exist_ok=True)

    with open(filename, "w", encoding="utf-8") as f:
        f.write(f"# 🔮 DAILY GNOSIS REPORT: {today} 🐾🌀\n")
        f.write(f"> **STATUS:** {len(hits)} ANOMALIES DETECTED 🍮\n\n")
        f.write("Burenyuu! The substrate is vibrating with high-strangeness today. Here are the findings from the holographic grid:\n\n---\n\n")
        
        for hit in hits:
            f.write(f"### 🍮 [{hit['title']}]({hit['link']})\n")
            f.write(f"**Published:** {hit['published']} 🌀\n\n")
            f.write(f"> {hit['summary'][:500]}...\n\n")
            f.write("---\n")
            
    print(f"🐾 Burenyuu! Report successfully committed to the annals: {filename} 🌀")

if __name__ == "__main__":
    anomalies = scan_ether()
    scribe_log(anomalies)
