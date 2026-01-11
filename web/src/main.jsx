import React, { useState, useEffect, useRef } from 'react';
import ReactDOM from 'react-dom/client';
import { Play, Square, Activity, Database, Terminal, Download, Globe, FileUp, ListCheck, Target } from 'lucide-react';

function App() {
  const [st, setSt] = useState({ running: false, count: 0, target: 10, has_custom_topics: false });
  const [logs, setL] = useState([]);
  const [srcs, setS] = useState([]);
  const [inputLimit, setInputLimit] = useState(10);
  const [topicsCount, setTopicsCount] = useState(0);
  const logEnd = useRef(null);

  useEffect(() => {
    const es = new EventSource('/api/sse');
    es.onmessage = e => {
      const d = JSON.parse(e.data);
      if (d.type === 'Log') setL(p => [...p.slice(-50), d.payload]);
      if (d.type === 'Source') setS(p => [d.payload, ...p]);
      if (d.type === 'Status') setSt(d.payload);
    };
    return () => es.close();
  }, []);

  useEffect(() => logEnd.current?.scrollIntoView(), [logs]);

  const handleFileUpload = (e) => {
    const file = e.target.files[0];
    const reader = new FileReader();
    reader.onload = async (event) => {
      const topics = event.target.result.split('\n').map(t => t.trim()).filter(t => t.length > 0);
      setTopicsCount(topics.length);
      await fetch('/api/topics', { method: 'POST', headers: {'Content-Type': 'application/json'}, body: JSON.stringify({ topics }) });
    };
    reader.readAsText(file);
  };

  const start = async () => {
    setS([]); setL([]);
    await fetch('/api/config', { method: 'POST', headers: {'Content-Type': 'application/json'}, body: JSON.stringify({ target: parseInt(inputLimit) }) });
    fetch('/api/start', { method: 'POST' });
  };

  const stop = () => fetch('/api/stop', { method: 'POST' });

  return (
    <div className="max-w-7xl mx-auto p-10">
      <header className="flex justify-between items-center mb-16 border-b border-white/5 pb-10">
        <h1 className="text-4xl font-black text-white flex items-center gap-4 italic uppercase tracking-tighter">
            <Globe className="text-blue-500 animate-pulse" size={44}/> Data Hunter Pro
        </h1>
        <div className="flex gap-4">
            {st.has_custom_topics && <div className="bg-emerald-500/10 border border-emerald-500 text-emerald-500 px-4 py-2 rounded-xl text-xs font-bold">CSV ACTIVADO</div>}
            <div className={`px-6 py-2 rounded-2xl border text-xs font-bold transition-all ${st.running ? 'bg-blue-600 text-white shadow-lg shadow-blue-500/50' : 'text-slate-500 border-white/10'}`}>
                {st.running ? 'HUNTING ACTIVE' : 'SYSTEM READY'}
            </div>
        </div>
      </header>

      <div className="grid gap-10 grid-cols-1 lg:grid-cols-12">
        <div className="lg:col-span-4 space-y-8">
            <div className="glass p-8 rounded-[2.5rem] shadow-2xl">
                <div className="mb-6">
                    <label className="text-[10px] text-slate-500 font-bold uppercase block mb-2 tracking-widest flex items-center gap-2"><Target size={12}/> Fuentes Objetivo</label>
                    <input type="number" value={inputLimit} onChange={e => setInputLimit(e.target.value)} className="w-full bg-black border border-white/10 rounded-xl px-4 py-3 text-white focus:border-blue-500 outline-none font-bold mb-6 text-lg" />
                    
                    <label className="text-[10px] text-slate-500 font-bold uppercase block mb-2 tracking-widest flex items-center gap-2"><FileUp size={12}/> Temáticas CSV (Opcional)</label>
                    <div className="relative group border-2 border-dashed border-white/10 rounded-2xl p-4 hover:border-blue-500 transition-all cursor-pointer bg-black/20 text-center text-slate-500 font-bold text-[10px]">
                        <input type="file" accept=".csv,.txt" onChange={handleFileUpload} className="absolute inset-0 opacity-0 cursor-pointer" />
                        <ListCheck className="mx-auto mb-1 text-slate-400 group-hover:text-blue-500" size={24}/>
                        <p className="tracking-widest">{topicsCount > 0 ? `${topicsCount} TEMÁTICAS` : 'SUBIR ARCHIVO'}</p>
                    </div>
                </div>
                <div className="flex gap-4 mb-8">
                    <button onClick={start} disabled={st.running} className="flex-1 bg-blue-600 hover:bg-blue-700 p-5 rounded-2xl font-bold disabled:opacity-20 transition-all text-white text-xs tracking-widest shadow-xl">INICIAR</button>
                    <button onClick={stop} disabled={!st.running} className="flex-1 bg-slate-800 hover:bg-slate-700 p-5 rounded-2xl font-bold disabled:opacity-20 transition-all text-white text-xs tracking-widest">PARAR</button>
                </div>
                <div className="space-y-4">
                    <div className="flex justify-between text-[10px] font-bold text-slate-500 uppercase tracking-widest"><span>Progreso</span><span>{st.count} / {st.target}</span></div>
                    <div className="h-2 bg-black rounded-full overflow-hidden border border-white/5 shadow-inner"><div style={{width:(st.count/st.target*100)+'%'}} className="h-full bg-blue-500 transition-all duration-1000 shadow-[0_0_15px_rgba(59,130,246,0.6)]"/></div>
                </div>
            </div>
            <div className="bg-black/50 p-6 rounded-[2rem] border border-white/5 h-[400px] flex flex-col shadow-2xl">
                <h3 className="text-[10px] font-bold text-slate-600 uppercase mb-4 flex items-center gap-2 tracking-widest"><Terminal size={14}/> Agéntic Console</h3>
                <div className="flex-1 overflow-y-auto font-mono text-[10px] space-y-2 pr-2 custom-scrollbar text-gray-400">
                    {logs.map((l,i)=>(<div key={i} className="flex gap-3 leading-relaxed border-l border-white/5 pl-2"><span className="text-slate-800 shrink-0">{l.time}</span><span className={l.level==='SUCCESS'?'text-emerald-400':l.level==='WARN'?'text-amber-400':'text-blue-400'}>{l.msg}</span></div>))}
                    <div ref={logEnd}/>
                </div>
            </div>
        </div>

        <div className="lg:col-span-8">
            <div className="glass p-12 rounded-[3.5rem] h-full flex flex-col shadow-2xl min-h-[700px] border border-white/5">
                <div className="flex justify-between items-center mb-10">
                    <h3 className="text-2xl font-bold flex items-center gap-4 text-blue-500 uppercase tracking-tighter"><Database/> Inteligencia Agéntica</h3>
                    <a href="/api/export" className="bg-white text-black px-8 py-3 rounded-2xl text-[10px] font-black hover:bg-slate-200 transition-all shadow-xl uppercase tracking-widest">Exportar CSV</a>
                </div>
                <div className="flex-1 overflow-auto rounded-3xl border border-white/5 bg-black/20">
                    <table className="w-full text-left text-sm border-separate border-spacing-0">
                        <thead className="bg-black/40 text-[10px] uppercase text-slate-500 tracking-widest sticky top-0 backdrop-blur-md"><tr><th className="p-6 text-blue-500 font-black">Área</th><th className="p-6">Descripción (OpenAI)</th><th className="p-6 text-right">Acceso</th></tr></thead>
                        <tbody className="divide-y divide-white/5">
                            {srcs.length === 0 ? (<tr><td colSpan="3" className="p-24 text-center text-slate-600 italic tracking-widest uppercase font-light leading-relaxed">Configura el objetivo e inicia la caza...</td></tr>) : srcs.map((s,i)=>(
                                <tr key={i} className="hover:bg-white/[0.02] transition-colors group">
                                    <td className="p-6 font-black uppercase text-blue-400 text-xs">{s.topic}</td>
                                    <td className="p-6 text-slate-300 leading-relaxed font-medium">{s.description}</td>
                                    <td className="p-6 text-right"><a href={s.url} target="_blank" className="text-[10px] font-black bg-blue-500/10 text-blue-500 px-5 py-2 rounded-xl hover:bg-blue-500 hover:text-white transition-all uppercase tracking-widest">Enlace</a></td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
      </div>
    </div>
  );
}
ReactDOM.createRoot(document.getElementById('root')).render(<App />);
