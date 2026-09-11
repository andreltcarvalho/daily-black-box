import {spawn} from 'node:child_process';import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';import assert from 'node:assert/strict';
const executable='src-tauri/target/debug/caixa-preta-native-host.exe';
const id=readFileSync('native-host/extension-id.txt','utf8').trim();
function frame(message){const payload=Buffer.from(JSON.stringify(message));const head=Buffer.alloc(4);head.writeUInt32LE(payload.length);return Buffer.concat([head,payload]);}
const child=spawn(executable,[`chrome-extension://${id}/`],{windowsHide:true,stdio:['pipe','pipe','pipe']});
let data=Buffer.alloc(0);const replies=[];const done=new Promise((resolve,reject)=>{
  const timer=setTimeout(()=>{child.kill();reject(new Error('Native host timeout'));},8000);
  child.on('error',reject);child.stderr.on('data',d=>console.error(String(d)));
  child.stdout.on('data',chunk=>{data=Buffer.concat([data,chunk]);while(data.length>=4&&data.length>=4+data.readUInt32LE(0)){const size=data.readUInt32LE(0);const reply=JSON.parse(data.subarray(4,4+size));data=data.subarray(4+size);replies.push(reply);if(replies.length===1)child.stdin.write(frame({kind:'poll'}));if(replies.length===2){clearTimeout(timer);child.stdin.end();resolve();}}});
  child.on('exit',code=>{if(replies.length<2){clearTimeout(timer);reject(new Error(`Native host exited ${code}`));}});
});
child.stdin.write(frame({kind:'hello',version:1}));await done;
assert.equal(replies[0].version,1);assert.equal(replies[0].paused,true);assert.equal(replies[1].kind,'tracking_state');
const denied=spawn(executable,['chrome-extension://invalid/'],{windowsHide:true,stdio:'ignore'});const code=await new Promise(resolve=>denied.on('exit',resolve));assert.equal(code,1);
mkdirSync('.cache',{recursive:true});writeFileSync('.cache/native-host-test.json',JSON.stringify({checks:['native framing round trip','same-user named pipe','paused state','denied origin'],passed:true},null,2));console.log('Native Messaging + named pipe: 4 checks passed.');
