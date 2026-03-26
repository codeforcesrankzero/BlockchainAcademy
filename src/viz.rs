use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{
    Arc, Mutex, OnceLock, RwLock,
    atomic::{AtomicUsize, Ordering},
    mpsc::{self, SyncSender, Receiver},
};
use std::thread;
use std::time::Duration;

const R:  &str = "\x1b[0m";
const B:  &str = "\x1b[1m";
const D:  &str = "\x1b[2m";
const RD: &str = "\x1b[31m";
const GR: &str = "\x1b[32m";
const YL: &str = "\x1b[33m";
const BL: &str = "\x1b[34m";
const MG: &str = "\x1b[35m";
const CY: &str = "\x1b[36m";

const PALETTE: [&str; 6] = [CY, GR, YL, MG, BL, RD];



const HTML_PAGE: &str = r#"<!DOCTYPE html>
<html lang="en"><head><meta charset="UTF-8">
<title>ToyChain Live</title>
<style>
*,*::before,*::after{box-sizing:border-box;margin:0;padding:0}
:root{--bg:#0d1117;--sur:#161b22;--brd:#30363d;--txt:#e6edf3;--mut:#8b949e;
  --blue:#58a6ff;--grn:#3fb950;--yel:#d29922;--red:#f85149;--pur:#bc8cff}
body{background:var(--bg);color:var(--txt);font-family:'Courier New',monospace;
  font-size:13px;height:100vh;display:flex;flex-direction:column;overflow:hidden}
header{background:var(--sur);border-bottom:1px solid var(--brd);
  padding:10px 16px;display:flex;align-items:center;gap:14px;flex-shrink:0}
header h1{font-size:14px;color:var(--blue)}
.pill{background:var(--bg);border:1px solid var(--brd);border-radius:20px;
  padding:2px 10px;font-size:12px}
.pill b{color:var(--blue)}
#dot{width:8px;height:8px;border-radius:50%;background:var(--grn);
  animation:pulse 2s ease-in-out infinite}
@keyframes pulse{0%,100%{opacity:1;transform:scale(1)}50%{opacity:.4;transform:scale(.8)}}
#chain{flex-shrink:0;border-bottom:1px solid var(--brd);overflow-x:auto;
  padding:14px 16px;min-height:125px;max-height:150px;
  display:flex;align-items:center;gap:0;
  scrollbar-width:thin;scrollbar-color:var(--brd) transparent}
.blk{flex-shrink:0;background:var(--sur);border:1px solid var(--brd);
  border-radius:8px;padding:8px 12px;width:100px;text-align:center;
  cursor:pointer;transition:border-color .2s,transform .2s;position:relative}
.blk:hover{border-color:var(--blue);transform:translateY(-2px)}
.blk.new{border-color:var(--grn)}
.blk.new::after{content:'';position:absolute;inset:-1px;border-radius:8px;
  border:1px solid var(--grn);animation:ring 1s ease-out forwards}
@keyframes ring{0%{opacity:1;transform:scale(1)}100%{opacity:0;transform:scale(1.2)}}
.blk-gen{background:#0d2137;border-color:#1f6feb}
.blk-h{font-size:20px;font-weight:bold;color:var(--blue);line-height:1}
.blk-hash{font-size:9px;color:var(--mut);margin:3px 0}
.blk-miner{font-size:11px;margin:2px 0}
.blk-txs{font-size:10px;color:var(--mut)}
.arrow{flex-shrink:0;color:var(--brd);font-size:18px;padding:0 5px;user-select:none}
.blk.sel{border-color:var(--blue)!important;background:#1c2e4a}
#detail{display:none;flex-shrink:0;background:#0d1c2e;border-bottom:1px solid var(--brd);
  padding:12px 20px;max-height:180px;overflow-y:auto;animation:slideDown .15s ease}
@keyframes slideDown{from{opacity:0;transform:translateY(-6px)}to{opacity:1;transform:none}}
.d-head{display:flex;justify-content:space-between;align-items:center;margin-bottom:6px}
.d-title{font-size:13px;font-weight:bold;color:var(--blue)}
.d-close{cursor:pointer;color:var(--mut);font-size:18px;line-height:1;padding:0 2px}
.d-close:hover{color:var(--txt)}
.d-meta{font-size:11px;color:var(--mut);margin-bottom:10px;font-family:'Courier New',monospace}
.d-txs{display:flex;flex-direction:column;gap:5px}
.d-tx{font-size:12px;padding:5px 12px;border-radius:5px;border-left:3px solid transparent;
  font-family:'Courier New',monospace}
.d-cb{border-color:var(--yel);background:rgba(210,153,34,.08)}
.d-ok{border-color:var(--grn);background:rgba(63,185,80,.08)}
.d-empty{color:var(--mut);font-size:12px;font-style:italic}
#bottom{flex:1;display:grid;grid-template-columns:260px 1fr;overflow:hidden}
#bals{background:var(--sur);border-right:1px solid var(--brd);
  padding:14px;overflow-y:auto;display:flex;flex-direction:column;gap:10px}
#bals h2{font-size:10px;text-transform:uppercase;letter-spacing:.1em;color:var(--mut)}
.bal-row{}
.bal-hdr{display:flex;justify-content:space-between;margin-bottom:4px}
.bal-name{font-size:12px;font-weight:bold}
.bal-amt{font-size:12px;color:var(--mut);transition:color .3s}
.bal-track{background:var(--bg);border-radius:4px;height:6px}
.bal-fill{height:6px;border-radius:4px;transition:width .5s cubic-bezier(.4,0,.2,1),
  background .4s;min-width:4px}
#log{padding:14px;overflow-y:auto;display:flex;flex-direction:column}
#log h2{font-size:10px;text-transform:uppercase;letter-spacing:.1em;
  color:var(--mut);margin-bottom:8px;flex-shrink:0}
#evs{display:flex;flex-direction:column;gap:2px}
.ev{font-size:12px;padding:4px 8px;border-radius:4px;
  border-left:3px solid transparent;
  white-space:nowrap;overflow:hidden;text-overflow:ellipsis;
  animation:fade .25s ease}
@keyframes fade{from{opacity:0;transform:translateX(-4px)}to{opacity:1;transform:none}}
.e-ok  {border-color:var(--grn);background:rgba(63,185,80,.06)}
.e-err {border-color:var(--red);background:rgba(248,81,73,.06)}
.e-blk {border-color:var(--blue);background:rgba(88,166,255,.06)}
.e-mine{border-color:var(--brd);color:var(--mut)}
.e-warn{border-color:var(--yel);background:rgba(210,153,34,.06)}
.e-reo {border-color:var(--pur);background:rgba(188,140,255,.1);font-weight:bold}
.e-gen {border-color:var(--blue);background:rgba(88,166,255,.1)}
.chain-lbl{font-size:10px;text-transform:uppercase;letter-spacing:.08em;
  padding:4px 16px 0;flex-shrink:0;display:none}
.chain-lbl.honest{color:var(--grn)}
.chain-lbl.attack{color:var(--yel)}
#chain-fork{flex-shrink:0;overflow-x:auto;padding:14px 16px;
  min-height:110px;max-height:130px;align-items:center;gap:0;
  display:none;border-bottom:1px solid var(--brd);
  scrollbar-width:thin;scrollbar-color:var(--brd) transparent}
.blk-atk{border-color:var(--yel)!important}
.blk-atk .blk-h{color:var(--yel)!important}
#fork-banner{display:none;flex-shrink:0;padding:9px 16px;text-align:center;
  font-weight:bold;font-size:13px;border-bottom:1px solid var(--brd)}
.fork-won{background:rgba(248,81,73,.15);color:var(--red)}
.fork-lost{background:rgba(63,185,80,.1);color:var(--grn)}
</style></head><body>
<header>
  <div id="dot"></div>
  <h1>⛓ ToyChain Live</h1>
  <div class="pill">Height <b id="s-h">—</b></div>
  <div class="pill">Blocks <b id="s-b">1</b></div>
  <div class="pill">Mempool <b id="s-m">0</b></div>
</header>
<div id="lbl-honest" class="chain-lbl honest">⚖ Honest Chain</div>
<div id="chain">
  <div class="blk blk-gen" id="blk-0">
    <div class="blk-h">#0</div>
    <div class="blk-hash">genesis</div>
    <div class="blk-miner" style="color:#1f6feb">⛓ origin</div>
    <div class="blk-txs">—</div>
  </div>
</div>
<div id="lbl-attack" class="chain-lbl attack">⚔ Attack Chain</div>
<div id="chain-fork"></div>
<div id="fork-banner"></div>
<div id="detail">
  <div class="d-head">
    <span class="d-title" id="d-title">Block</span>
    <span class="d-close" onclick="closeDetail()">✕</span>
  </div>
  <div class="d-meta" id="d-meta"></div>
  <div class="d-txs" id="d-txs"></div>
</div>
<div id="bottom">
  <div id="bals"><h2>💰 Balances</h2></div>
  <div id="log">
    <h2>📋 Events</h2>
    <div id="evs"></div>
  </div>
</div>
<script>
const WEB_PALETTE=['#58a6ff','#3fb950','#d29922','#bc8cff','#79c0ff','#db61a2'];
const actors={};let cidx=0;
const bals={};let maxBal=1;let mp=0;
const blockData={};
let selectedBlock=null;
let forkMode=false;

function color(addr){return(actors[addr]||{}).color||'#8b949e'}
function name(addr){
  if(!addr)return'?';
  const a=actors[addr];
  return a?a.name:addr.slice(0,6)+'..';
}
function reg(addr,n){
  const c=WEB_PALETTE[cidx++%WEB_PALETTE.length];
  actors[addr]={name:n,color:c};
  actors[n]={name:n,color:c};
}

function addBlock(h,hash,miner,txn){
  const sec=document.getElementById('chain');
  sec.querySelectorAll('.blk.new').forEach(b=>b.classList.remove('new'));
  const arr=document.createElement('span');
  arr.className='arrow';arr.textContent='→';sec.appendChild(arr);
  const blk=document.createElement('div');
  blk.className='blk new';blk.id='blk-'+h;
  blk.title='Click to see transactions';
  blk.innerHTML=
    '<div class="blk-h">#'+h+'</div>'+
    '<div class="blk-hash">'+hash+'…</div>'+
    '<div class="blk-miner" style="color:'+color(miner)+'">'+name(miner)+'</div>'+
    '<div class="blk-txs">'+txn+' tx</div>';
  blk.onclick=()=>showDetail(h);
  sec.appendChild(blk);
  sec.scrollLeft=sec.scrollWidth;
  document.getElementById('s-h').textContent=h;
  document.getElementById('s-b').textContent=h+1;
}

function showDetail(h){
  if(selectedBlock===h){closeDetail();return;}
  selectedBlock=h;
  document.querySelectorAll('.blk').forEach(b=>b.classList.remove('sel'));
  const blk=document.getElementById('blk-'+h);
  if(blk)blk.classList.add('sel');
  const bd=blockData[h];
  document.getElementById('d-title').textContent='Block #'+h;
  if(!bd){
    document.getElementById('d-meta').textContent='loading…';
    document.getElementById('d-txs').innerHTML='';
  } else {
    document.getElementById('d-meta').textContent=
      'hash: '+bd.hash+'…   miner: '+name(bd.miner);
    const txsEl=document.getElementById('d-txs');
    if(!bd.txs||bd.txs.length===0){
      txsEl.innerHTML='<div class="d-empty">no transactions</div>';
    } else {
      txsEl.innerHTML=bd.txs.map(tx=>{
        if(tx.is_coinbase)
          return '<div class="d-tx d-cb">⛏  coinbase  →  '+
            '<span style="color:'+color(tx.to)+'">'+name(tx.to)+'</span>'+
            '  <b>+'+tx.amount+'</b></div>';
        return '<div class="d-tx d-ok">✓  '+
          '<span style="color:'+color(tx.from)+'">'+name(tx.from)+'</span>'+
          '  →  '+
          '<span style="color:'+color(tx.to)+'">'+name(tx.to)+'</span>'+
          '  <b>'+tx.amount+'</b></div>';
      }).join('');
    }
  }
  document.getElementById('detail').style.display='block';
}

function closeDetail(){
  document.getElementById('detail').style.display='none';
  document.querySelectorAll('.blk').forEach(b=>b.classList.remove('sel'));
  selectedBlock=null;
}

document.getElementById('blk-0').onclick=()=>showDetail(0);
blockData[0]={hash:'0000000000000000',miner:'',txs:[]};

function addGhosts(count){
  const sec=document.getElementById('chain-fork');
  for(let i=0;i<count;i++){
    if(i>0){
      const arr=document.createElement('span');
      arr.className='arrow';arr.style.opacity='0.12';arr.textContent='→';
      sec.appendChild(arr);
    }
    const blk=document.createElement('div');
    blk.className='blk';
    blk.style.cssText='opacity:0.1;pointer-events:none;border-style:dashed;flex-shrink:0';
    sec.appendChild(blk);
  }
}

function activateFork(h){
  if(forkMode)return;
  forkMode=true;
  addGhosts(h);
  const pivot=document.getElementById('blk-'+(h-1));
  if(pivot){
    pivot.style.borderColor='var(--yel)';
    const tag=document.createElement('div');
    tag.style.cssText='font-size:9px;color:var(--yel);text-align:center;margin-top:3px;letter-spacing:.03em';
    tag.textContent='⬇ fork';
    pivot.appendChild(tag);
  }
  document.getElementById('lbl-honest').style.display='block';
  document.getElementById('lbl-attack').style.display='block';
  document.getElementById('chain-fork').style.display='flex';
  const c=document.getElementById('chain'),f=document.getElementById('chain-fork');
  c.addEventListener('scroll',()=>{f.scrollLeft=c.scrollLeft;},{passive:true});
  f.addEventListener('scroll',()=>{c.scrollLeft=f.scrollLeft;},{passive:true});
}

function addForkBlock(h,hash,miner,txn){
  activateFork(h);
  const sec=document.getElementById('chain-fork');
  const arr=document.createElement('span');
  arr.className='arrow';arr.textContent='→';sec.appendChild(arr);
  const blk=document.createElement('div');
  blk.className='blk blk-atk new';blk.id='fblk-'+h;
  blk.innerHTML=
    '<div class="blk-h">#'+h+'</div>'+
    '<div class="blk-hash">'+hash+'…</div>'+
    '<div class="blk-miner" style="color:'+color(miner)+'">'+name(miner)+'</div>'+
    '<div class="blk-txs">'+txn+' tx</div>';
  sec.appendChild(blk);
  sec.scrollLeft=sec.scrollWidth;
}

function showForkOutcome(won,nh,ah){
  const el=document.getElementById('fork-banner');
  el.style.display='block';
  if(won){
    el.className='fork-won';
    el.textContent='⚠ ATTACK SUCCESSFUL — attacker chain (#'+ah+') overtook honest (#'+nh+')';
  } else {
    el.className='fork-lost';
    el.textContent='✓ ATTACK FAILED — honest chain (#'+nh+') is longer than attacker (#'+ah+')';
  }
}

function updBal(addr,val){
  const n=name(addr),c=color(addr),old=bals[n]||0;
  bals[n]=val;
  maxBal=Math.max(...Object.values(bals),1);
  let row=document.getElementById('bal-'+n);
  if(!row){
    row=document.createElement('div');row.className='bal-row';row.id='bal-'+n;
    row.innerHTML=
      '<div class="bal-hdr">'+
        '<span class="bal-name" style="color:'+c+'">'+n+'</span>'+
        '<span class="bal-amt" id="ba-'+n+'">0</span>'+
      '</div>'+
      '<div class="bal-track"><div class="bal-fill" id="bf-'+n+
        '" style="background:'+c+';width:0%"></div></div>';
    document.getElementById('bals').appendChild(row);
  }
  document.getElementById('ba-'+n).textContent=val;
  for(const[k,v]of Object.entries(bals)){
    const el=document.getElementById('bf-'+k);
    if(el)el.style.width=(v/maxBal*100).toFixed(1)+'%';
  }
  const fill=document.getElementById('bf-'+n);
  if(fill){
    const flash=val>old?'#3fb950':'#f85149';
    fill.style.transition='none';fill.style.background=flash;
    setTimeout(()=>{fill.style.transition='width .5s cubic-bezier(.4,0,.2,1),background .5s';fill.style.background=c;},40);
  }
}

function log(cls,txt){
  const el=document.createElement('div');el.className='ev '+cls;el.textContent=txt;
  const evs=document.getElementById('evs');
  evs.insertBefore(el,evs.firstChild);
  while(evs.children.length>300)evs.removeChild(evs.lastChild);
}

const es=new EventSource('/events');
es.onmessage=e=>{
  try{
    const d=JSON.parse(e.data);
    if(d.type==='register'){reg(d.addr,d.name);}
    else if(d.type==='genesis'){
      document.querySelector('#blk-0 .blk-hash').textContent=d.hash+'…';
      log('e-gen','⛓  genesis  '+d.hash+'…');
    }
    else if(d.type==='block_mined'){
      log('e-mine','⛏  #'+d.height+'  miner='+name(d.miner)+'  nonce='+d.nonce);
    }
    else if(d.type==='block_added'){
      blockData[d.height]={hash:d.hash,miner:d.miner,txs:d.txs||[]};
      if(selectedBlock===d.height)showDetail(d.height);
      if(d.chain==='attack'){
        addForkBlock(d.height,d.hash,d.miner,d.tx_count);
        log('e-warn','⚔  Fork #'+d.height+'  '+name(d.miner)+'  '+d.tx_count+' txs');
      } else {
        addBlock(d.height,d.hash,d.miner,d.tx_count);
        const reg_txs=Math.max(0,d.tx_count-1);
        mp=Math.max(0,mp-reg_txs);
        document.getElementById('s-m').textContent=mp;
        log('e-blk','✓  Block #'+d.height+'  '+name(d.miner)+'  '+d.tx_count+' txs');
      }
    }
    else if(d.type==='fork_outcome'){
      showForkOutcome(d.attacker_won,d.honest_height,d.attack_height);
      if(d.attacker_won)
        log('e-err','⚠ ATTACK WON  attacker=#'+d.attack_height+'  honest=#'+d.honest_height);
      else
        log('e-ok','✓ ATTACK FAILED  honest=#'+d.honest_height+'  attacker=#'+d.attack_height);
    }
    else if(d.type==='block_rejected'){
      log('e-err','✗  Block #'+d.height+': '+d.reason);
    }
    else if(d.type==='tx_accepted'){
      mp++;document.getElementById('s-m').textContent=mp;
      if(d.warn_dup)
        log('e-warn','⚠  '+name(d.from)+' → '+name(d.to)+'  '+d.amount+'  nonce='+d.nonce+'  dup nonce!');
      else
        log('e-ok','✓  '+name(d.from)+' → '+name(d.to)+'  '+d.amount+'  nonce='+d.nonce);
    }
    else if(d.type==='tx_rejected'){
      log('e-err','✗  '+name(d.from)+' → '+name(d.to)+'  '+d.amount+'  '+d.reason);
    }
    else if(d.type==='balance'){
      updBal(d.addr,d.new_bal);
    }
    else if(d.type==='reorg'){
      log('e-reo','⚡  REORG  fork@'+d.fork_height+'  '+d.old_height+' → '+d.new_height);
    }
  }catch(err){console.error('viz:',err);}
};
es.onerror=()=>{
  const d=document.getElementById('dot');
  d.style.background='#f85149';d.style.animation='none';
};
</script></body></html>
"#;


struct NameEntry { name: String, color_idx: usize }

static REGISTRY:   OnceLock<RwLock<HashMap<String, NameEntry>>> = OnceLock::new();
static REG_COUNT:  AtomicUsize = AtomicUsize::new(0);

fn registry() -> &'static RwLock<HashMap<String, NameEntry>> {
    REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

pub fn register(addr: &str, label: &str) {
    let idx = REG_COUNT.fetch_add(1, Ordering::Relaxed);
    registry().write().unwrap().insert(
        addr.to_string(),
        NameEntry { name: label.to_string(), color_idx: idx },
    );
    broadcast_sse(&format!(
        r#"{{"type":"register","addr":"{}","name":"{}"}}"#,
        addr, label
    ));
}


#[derive(Clone, Debug)]
pub struct TxInfo {
    pub from:        String,
    pub to:          String,
    pub amount:      u64,
    pub is_coinbase: bool,
}

#[derive(Clone, Debug)]
pub enum VizEvent {
    Genesis      { hash_prefix: String },
    BlockMined   { height: u64, miner: String, tx_count: usize, nonce: u64 },
    BlockAdded   { height: u64, hash_prefix: String, miner: String, txs: Vec<TxInfo> },
    BlockRejected{ height: u64, reason: String },
    Reorg        { fork_height: u64, old_height: u64, new_height: u64 },
    TxAccepted   { from: String, to: String, amount: u64, nonce: u64, warn_dup_nonce: bool },
    TxRejected   { from: String, to: String, amount: u64, reason: String },
    BalanceChange{ addr: String, old_bal: u128, new_bal: u128 },
    ForkOutcome  { attacker_won: bool, honest_height: u64, attack_height: u64 },
}


static SENDER: OnceLock<SyncSender<VizEvent>> = OnceLock::new();

pub fn emit(event: VizEvent) {
    if let Some(tx) = SENDER.get() { let _ = tx.try_send(event); }
}

pub fn active() -> bool { SENDER.get().is_some() }


static CHAIN_LABEL: OnceLock<Mutex<String>> = OnceLock::new();

pub fn set_chain(label: &str) {
    *CHAIN_LABEL.get_or_init(|| Mutex::new("main".to_string()))
        .lock().unwrap() = label.to_string();
}

fn current_chain() -> String {
    CHAIN_LABEL.get_or_init(|| Mutex::new("main".to_string()))
        .lock().unwrap()
        .clone()
}


static HISTORY:     OnceLock<Arc<Mutex<Vec<String>>>>           = OnceLock::new();
static SSE_CLIENTS: OnceLock<Arc<Mutex<Vec<TcpStream>>>>        = OnceLock::new();

fn history()     -> &'static Arc<Mutex<Vec<String>>>     { HISTORY.get_or_init(Default::default) }
fn sse_clients() -> &'static Arc<Mutex<Vec<TcpStream>>>  { SSE_CLIENTS.get_or_init(Default::default) }
.
pub fn start() {
    let (tx, rx) = mpsc::sync_channel::<VizEvent>(50_000);
    if SENDER.set(tx).is_err() { return; }

    match TcpListener::bind("127.0.0.1:3000") {
        Ok(listener) => {
            println!("\n{}{}🌐  Visualizer → http://localhost:3000{}\n", B, CY, R);
            thread::spawn(move || {
                for stream in listener.incoming().flatten() {
                    let h = history().clone();
                    let c = sse_clients().clone();
                    thread::spawn(move || handle_http(stream, h, c));
                }
            });
        }
        Err(_) => eprintln!("viz: port 3000 busy — web dashboard disabled"),
    }

    thread::spawn(move || render(rx));
    thread::sleep(Duration::from_millis(10));
}

pub fn flush() {
    thread::sleep(Duration::from_millis(200));
}


fn handle_http(
    mut stream: TcpStream,
    history:    Arc<Mutex<Vec<String>>>,
    clients:    Arc<Mutex<Vec<TcpStream>>>,
) {
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
    let mut buf = [0u8; 256];
    let n = stream.read(&mut buf).unwrap_or(0);
    let req = std::str::from_utf8(&buf[..n]).unwrap_or("");
    stream.set_read_timeout(None).ok();

    if req.contains("GET /events") {
        let _ = stream.write_all(
            b"HTTP/1.1 200 OK\r\n\
              Content-Type: text/event-stream\r\n\
              Cache-Control: no-cache\r\n\
              Access-Control-Allow-Origin: *\r\n\
              Connection: keep-alive\r\n\r\n",
        );
        {
            let hist = history.lock().unwrap();
            for json in hist.iter() {
                if stream.write_all(format!("data: {}\n\n", json).as_bytes()).is_err() {
                    return;
                }
            }
        }
        if stream.flush().is_err() { return; }
        clients.lock().unwrap().push(stream);
    } else {
        let _ = stream.write_all(
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\
                 Content-Length: {}\r\nConnection: close\r\n\r\n{}",
                HTML_PAGE.len(),
                HTML_PAGE
            )
            .as_bytes(),
        );
    }
}


fn broadcast_sse(json: &str) {
    history().lock().unwrap().push(json.to_string());
    let msg = format!("data: {}\n\n", json);
    let mut cls = sse_clients().lock().unwrap();
    cls.retain_mut(|s| s.write_all(msg.as_bytes()).is_ok() && s.flush().is_ok());
}

fn event_to_json(ev: &VizEvent) -> String {
    match ev {
        VizEvent::Genesis { hash_prefix } =>
            format!(r#"{{"type":"genesis","hash":"{}"}}"#, hash_prefix),

        VizEvent::BlockMined { height, miner, tx_count, nonce } =>
            format!(r#"{{"type":"block_mined","height":{},"miner":"{}","tx_count":{},"nonce":{}}}"#,
                height, miner, tx_count, nonce),

        VizEvent::BlockAdded { height, hash_prefix, miner, txs } => {
            let txs_json: String = txs.iter().map(|t| {
                format!(r#"{{"from":"{}","to":"{}","amount":{},"is_coinbase":{}}}"#,
                    t.from, t.to, t.amount, t.is_coinbase)
            }).collect::<Vec<_>>().join(",");
            format!(r#"{{"type":"block_added","chain":"{}","height":{},"hash":"{}","miner":"{}","tx_count":{},"txs":[{}]}}"#,
                current_chain(), height, hash_prefix, miner, txs.len(), txs_json)
        }

        VizEvent::BlockRejected { height, reason } =>
            format!(r#"{{"type":"block_rejected","height":{},"reason":"{}"}}"#,
                height, reason.replace('"', "'")),

        VizEvent::Reorg { fork_height, old_height, new_height } =>
            format!(r#"{{"type":"reorg","fork_height":{},"old_height":{},"new_height":{}}}"#,
                fork_height, old_height, new_height),

        VizEvent::TxAccepted { from, to, amount, nonce, warn_dup_nonce } =>
            format!(r#"{{"type":"tx_accepted","from":"{}","to":"{}","amount":{},"nonce":{},"warn_dup":{}}}"#,
                from, to, amount, nonce, warn_dup_nonce),

        VizEvent::TxRejected { from, to, amount, reason } =>
            format!(r#"{{"type":"tx_rejected","from":"{}","to":"{}","amount":{},"reason":"{}"}}"#,
                from, to, amount, reason.replace('"', "'")),

        VizEvent::BalanceChange { addr, old_bal, new_bal } =>
            format!(r#"{{"type":"balance","addr":"{}","old_bal":{},"new_bal":{}}}"#,
                addr, old_bal, new_bal),

        VizEvent::ForkOutcome { attacker_won, honest_height, attack_height } =>
            format!(r#"{{"type":"fork_outcome","attacker_won":{},"honest_height":{},"attack_height":{}}}"#,
                attacker_won, honest_height, attack_height),
    }
}


fn short(s: &str) -> String {
    let s = s.trim_start_matches("0x");
    if s.len() >= 12 { format!("{}..{}", &s[..5], &s[s.len()-4..]) } else { s.to_string() }
}

fn display(addr: &str) -> String {
    if addr.is_empty() { return format!("{}coinbase{}", D, R); }
    if let Ok(reg) = registry().read() {
        if let Some(e) = reg.get(addr) {
            let c = PALETTE[e.color_idx % PALETTE.len()];
            return format!("{}{}{}{}", B, c, e.name, R);
        }
    }
    let h = addr.bytes().fold(0usize, |a, b| a.wrapping_mul(31).wrapping_add(b as usize));
    format!("{}{}{}", PALETTE[h % PALETTE.len()], short(addr), R)
}

fn pad(s: &str, w: usize) -> String {
    let vis: usize = {
        let mut n = 0; let mut esc = false;
        for c in s.chars() {
            if c == '\x1b' { esc = true; continue; }
            if esc { if c == 'm' { esc = false; } continue; }
            n += 1;
        }
        n
    };
    if vis >= w { s.to_string() } else { format!("{}{}", s, " ".repeat(w - vis)) }
}

fn chain_bar(height: u64) -> String {
    let start = height.saturating_sub(5);
    let mut out = String::new();
    if start > 0 { out.push_str(&format!("{}…━━{}", D, R)); }
    for i in start..=height {
        if i > start || start > 0 { out.push_str(&format!("{}━━{}", D, R)); }
        if i == height { out.push_str(&format!("{}{}[#{}]{}", B, CY, i, R)); }
        else           { out.push_str(&format!("{}[#{}]{}", D, i, R)); }
    }
    out
}


fn render(rx: Receiver<VizEvent>) {
    while let Ok(ev) = rx.recv() {
        broadcast_sse(&event_to_json(&ev));
        match &ev {
            VizEvent::Genesis { hash_prefix } => {
                println!();
                println!("{}{}╔══ GENESIS ════════════════════════════════════════╗{}", B, BL, R);
                println!("{}║{}  ⛓  chain initialized  {}hash={}…{}  {}║{}", BL, R, D, hash_prefix, R, BL, R);
                println!("{}╚══════════════════════════════════════════════════╝{}", BL, R);
            }
            VizEvent::BlockMined { height, miner, tx_count, nonce } => {
                println!("{}  ⛏  mining…{}  #{}  miner={}  txs={}  nonce={}",
                    D, R, height, display(miner), tx_count, nonce);
            }
            VizEvent::BlockAdded { height, hash_prefix, miner, txs } => {
                let bar = chain_bar(*height);
                println!();
                println!("{}{}┌─[#{}]─ BLOCK ADDED ─────────────────────────────────{}", B, GR, height, R);
                println!("{}│{}  {}  {}hash={}…{}", GR, R, display(miner), D, hash_prefix, R);
                for tx in txs {
                    if tx.is_coinbase {
                        println!("{}│{}  {}⛏  coinbase{}  →  {}  {}+{}{}", GR, R, D, R, display(&tx.to), GR, tx.amount, R);
                    } else {
                        println!("{}│{}  {}✓{}  {}  →  {}  {}", GR, R, GR, R,
                            pad(&display(&tx.from), 14), display(&tx.to), tx.amount);
                    }
                }
                println!("{}│{}  {}", GR, R, bar);
                println!("{}└──────────────────────────────────────────────────────{}", GR, R);
            }
            VizEvent::BlockRejected { height, reason } => {
                println!("  {}{}✗ Block #{} REJECTED:{} {}{}{}", B, RD, height, R, RD, reason, R);
            }
            VizEvent::Reorg { fork_height, old_height, new_height } => {
                println!();
                println!("{}{}⚡ CHAIN REORG{}  fork@{}  {}{}→{}{}", B, YL, R, fork_height, YL, old_height, new_height, R);
            }
            VizEvent::TxAccepted { from, to, amount, nonce, warn_dup_nonce } => {
                let f = pad(&display(from), 14);
                let t = display(to);
                if *warn_dup_nonce {
                    println!("  {}⚠  mempool{}  {}  →  {}  {}  {}nonce={}  ← dup nonce!{}",
                        YL, R, f, t, amount, YL, nonce, R);
                } else {
                    println!("  {}✓  mempool{}  {}  →  {}  {}  {}nonce={}{}",
                        GR, R, f, t, amount, D, nonce, R);
                }
            }
            VizEvent::TxRejected { from, to, amount, reason } => {
                let f = pad(&display(from), 14);
                let t = display(to);
                println!("  {}✗  mempool{}  {}  →  {}  {} coins  {}{}{}", RD, R, f, t, amount, RD, reason, R);
            }
            VizEvent::BalanceChange { addr, old_bal, new_bal } => {
                if old_bal == new_bal { continue; }
                let a = pad(&display(addr), 14);
                if new_bal > old_bal {
                    println!("     {}💰 {}  {} → {}{}{}  {}(+{}){}", D, a, old_bal, GR, new_bal, R, D, new_bal - old_bal, R);
                } else {
                    println!("     {}💰 {}  {} → {}{}{}  {}(-{}){}", D, a, old_bal, RD, new_bal, R, D, old_bal - new_bal, R);
                }
            }
            VizEvent::ForkOutcome { attacker_won, honest_height, attack_height } => {
                println!();
                if *attacker_won {
                    println!("{}{}⚠  ATTACK SUCCESSFUL{}  attacker=#{}  honest=#{}  (double-spend succeeded)",
                        B, RD, R, attack_height, honest_height);
                } else {
                    println!("{}{}✓  ATTACK FAILED{}  honest=#{}  attacker=#{}  (honest chain longer)",
                        B, GR, R, honest_height, attack_height);
                }
            }
        }
    }
}
