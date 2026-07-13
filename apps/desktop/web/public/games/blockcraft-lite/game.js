(()=>{
const c=document.querySelector('#gl'),gl=c.getContext('webgl',{antialias:false}),menu=document.querySelector('#menu'),play=document.querySelector('#play'),hot=document.querySelector('#hotbar'),pos=document.querySelector('#pos');
if(!gl){menu.innerHTML='WebGL indisponible';return}
const N=48,Y=24,SAVE='blockcraft-world-v2',types=['HERBE','TERRE','PIERRE','BOIS','BRIQUE','CRISTAL','EAU'],placeTypes=[1,2,3,4,5,6,8];
let world=new Uint8Array(N*N*Y),mesh=0,selected=0,px=N/2,py=12,pz=N/2,yaw=.4,pitch=-.15,vy=0,onGround=false,last=0,flowTimer=0,fly=false,keys={},locked=false;
const idx=(x,y,z)=>x+N*(z+N*y),inb=(x,y,z)=>x>=0&&z>=0&&y>=0&&x<N&&z<N&&y<Y,get=(x,y,z)=>inb(x,y,z)?world[idx(x,y,z)]:0,set=(x,y,z,v)=>{if(inb(x,y,z))world[idx(x,y,z)]=v};
const rnd=(x,z)=>{let n=(x*374761393+z*668265263)^0x5bf03635;n=(n^(n>>13))*1274126177;return ((n^(n>>16))>>>0)/4294967295};
function heightAt(x,z){let h=8+Math.sin(x*.23)*2.1+Math.cos(z*.19)*2.4+Math.sin((x+z)*.11)*1.7+(rnd(x,z)-.5)*2.2;return Math.max(3,Math.min(Y-6,Math.floor(h)))}
function tree(x,y,z){if(y>Y-7)return;for(let i=1;i<5;i++)set(x,y+i,z,4);for(let dx=-2;dx<=2;dx++)for(let dz=-2;dz<=2;dz++)for(let dy=3;dy<=6;dy++){let d=Math.abs(dx)+Math.abs(dz)+(dy===6?1:0);if(d<4)set(x+dx,y+dy,z+dz,7)}}
function generate(){world.fill(0);for(let x=0;x<N;x++)for(let z=0;z<N;z++){let h=heightAt(x,z);for(let y=0;y<=h;y++)set(x,y,z,y===h?1:y>h-3?2:3);if(h<7)for(let y=h+1;y<=7;y++)set(x,y,z,8);if(x>3&&z>3&&x<N-4&&z<N-4&&rnd(x,z)>.965)tree(x,h,z);if(rnd(x+99,z-31)>.986)set(x,h+1,z,6)}let h=heightAt(N/2,N/2);px=N/2+.5;py=h+3;pz=N/2+.5;try{localStorage.setItem(SAVE,JSON.stringify([...world]))}catch{}}
try{let s=localStorage.getItem(SAVE),a=s&&JSON.parse(s);if(Array.isArray(a)&&a.length===world.length)world.set(a);else generate()}catch{generate()}if(!world.some(v=>v))generate();
const vs=`attribute vec3 p;attribute vec3 co;uniform vec3 cam;uniform float yaw,pitch,aspect;varying vec3 v;void main(){vec3 d=p-cam;float sy=sin(yaw),cy=cos(yaw),sp=sin(pitch),cp=cos(pitch);vec3 right=vec3(cy,0.,sy);vec3 forward=vec3(sy*cp,sp,-cy*cp);vec3 up=vec3(-sy*sp,cp,cy*sp);float cx=dot(d,right),cY=dot(d,up),cz=dot(d,forward);float t=.52,near=.08,far=160.;float da=(far+near)/(far-near),db=(-2.*far*near)/(far-near);v=co;gl_Position=vec4(cx/(t*aspect),cY/t,da*cz+db,cz);}`;
const fs=`precision mediump float;varying vec3 v;void main(){gl_FragColor=vec4(v,1.);}`;
function sh(t,s){let q=gl.createShader(t);gl.shaderSource(q,s);gl.compileShader(q);if(!gl.getShaderParameter(q,gl.COMPILE_STATUS))throw gl.getShaderInfoLog(q);return q}
let pr=gl.createProgram();gl.attachShader(pr,sh(gl.VERTEX_SHADER,vs));gl.attachShader(pr,sh(gl.FRAGMENT_SHADER,fs));gl.linkProgram(pr);gl.useProgram(pr);
let ap=gl.getAttribLocation(pr,'p'),ac=gl.getAttribLocation(pr,'co'),uc=gl.getUniformLocation(pr,'cam'),uyaw=gl.getUniformLocation(pr,'yaw'),upitch=gl.getUniformLocation(pr,'pitch'),uaspect=gl.getUniformLocation(pr,'aspect'),buf=gl.createBuffer();
const faces=[[[0,0,0],[0,1,0],[1,1,0],[1,0,0],[0,0,-1]],[[1,0,0],[1,1,0],[1,1,1],[1,0,1],[1,0,0]],[[1,0,1],[1,1,1],[0,1,1],[0,0,1],[0,0,1]],[[0,0,1],[0,1,1],[0,1,0],[0,0,0],[-1,0,0]],[[0,1,0],[0,1,1],[1,1,1],[1,1,0],[0,1,0]],[[0,0,1],[0,0,0],[1,0,0],[1,0,1],[0,-1,0]]];
const base={1:[.25,.66,.25],2:[.47,.29,.14],3:[.43,.45,.47],4:[.52,.31,.13],5:[.70,.25,.18],6:[.20,.82,.96],7:[.18,.56,.20],8:[.16,.42,.88],9:[.12,.52,.95]};
const isWater=t=>t===8||t===9;
function tex(t,x,y,z,k,v){let b=base[t]||[1,0,1],n=rnd(x*7+v*3+y,z*5+k*11),l=k===4?1.18:k===5?.56:.82;if(t===1&&k!==4)b=[.39,.25,.12];if(t===4)b=(x+z+y+v)%2?[.58,.36,.16]:[.38,.22,.10];if(t===3)b=n>.55?[.50,.52,.54]:[.34,.36,.39];if(t===5)b=(x+y+z+v)%3?[.70,.25,.18]:[.45,.13,.10];if(t===6)l*=1.15+n*.35;if(isWater(t)){b=n>.45?[.10,.48,.95]:[.20,.76,1.0];l=k===4?1.05:.68}let grain=(n-.5)*(isWater(t)?.09:.16);return [Math.min(1,b[0]*l+grain),Math.min(1,b[1]*l+grain),Math.min(1,b[2]*l+grain)]}
function waterY(x,y,z,v){return y+.78+Math.sin((x*1.7+z*1.3+v)*2.1)*.035}
function build(){let a=[];for(let y=0;y<Y;y++)for(let z=0;z<N;z++)for(let x=0;x<N;x++){let t=get(x,y,z);if(!t)continue;faces.forEach((f,k)=>{let n=f[4],nb=get(x+n[0],y+n[1],z+n[2]);if(nb&&(!isWater(t)||isWater(nb)))return;let q=f.slice(0,4);[0,1,2,0,2,3].forEach((j,ii)=>{let vx=q[j][0],vy=q[j][1],vz=q[j][2];if(isWater(t)&&vy===1)vy=waterY(x,y,z,j+ii)-y;let cc=tex(t,x,y,z,k,j+ii);a.push(x+vx,y+vy,z+vz,cc[0],cc[1],cc[2])})})}gl.bindBuffer(gl.ARRAY_BUFFER,buf);gl.bufferData(gl.ARRAY_BUFFER,new Float32Array(a),gl.STATIC_DRAW);mesh=a.length/6;try{localStorage.setItem(SAVE,JSON.stringify([...world]))}catch{}}
build();
function ray(){let dx=Math.sin(yaw)*Math.cos(pitch),dy=Math.sin(pitch),dz=-Math.cos(yaw)*Math.cos(pitch),prev=null;for(let d=0;d<8;d+=.06){let X=Math.floor(px+dx*d),Y0=Math.floor(py+dy*d),Z=Math.floor(pz+dz*d),b=get(X,Y0,Z);if(b&&!isWater(b))return{hit:[X,Y0,Z],prev};prev=[X,Y0,Z]}return null}
function hud(){hot.innerHTML=types.map((n,i)=>`<span class="slot ${i===selected?'sel':''}">${i+1}<br>${n}</span>`).join('');pos.textContent=`${px.toFixed(1)} · ${py.toFixed(1)} · ${pz.toFixed(1)} | ${fly?'VOL ON':'SOL'} | F: vol · R: nouveau monde`}
function collide(nx,ny,nz){for(let dx of[-.28,.28])for(let dz of[-.28,.28])for(let yy of[ny-1.68,ny-.08]){let b=get(Math.floor(nx+dx),Math.floor(yy),Math.floor(nz+dz));if(b&&!isWater(b))return true}return false}
function feet(nx,ny,nz){for(let dx of[-.28,.28])for(let dz of[-.28,.28]){let b=get(Math.floor(nx+dx),Math.floor(ny-1.72),Math.floor(nz+dz));if(b&&!isWater(b))return true}return false}
function inside(x,y,z){return x<px+.28&&x+1>px-.28&&z<pz+.28&&z+1>pz-.28&&y<py&&y+1>py-1.7}
function flowWater(limit=90){
  let changes=[],seen=new Set(),add=(x,y,z,v=9)=>{let k=x+','+y+','+z;if(changes.length<limit&&!seen.has(k)&&inb(x,y,z)&&!get(x,y,z)){seen.add(k);changes.push([x,y,z,v])}};
  for(let y=Y-2;y>=1&&changes.length<limit;y--)for(let z=1;z<N-1&&changes.length<limit;z++)for(let x=1;x<N-1&&changes.length<limit;x++){
    let t=get(x,y,z);if(!isWater(t))continue;
    if(!get(x,y-1,z)){add(x,y-1,z,9);continue}
    if(t!==8)continue;
    let dirs=[[1,0],[-1,0],[0,1],[0,-1]].sort(()=>Math.random()-.5),spread=0;
    for(let [dx,dz] of dirs){
      if(spread>=2)break;
      if(!get(x+dx,y,z+dz)){
        add(x+dx,y,z+dz,9);spread++;
      }
    }
  }
  if(!changes.length)return;changes.forEach(p=>set(p[0],p[1],p[2],p[3]));build()
}
addEventListener('keydown',e=>{keys[e.key.toLowerCase()]=true;if(+e.key>=1&&+e.key<=types.length){selected=+e.key-1;hud()}if(e.code==='Space')e.preventDefault();if(e.key.toLowerCase()==='f'){fly=!fly;vy=0;hud()}if(e.key.toLowerCase()==='r'&&confirm('Generer un nouveau monde ?')){generate();build();hud()}});
addEventListener('keyup',e=>keys[e.key.toLowerCase()]=false);c.oncontextmenu=e=>e.preventDefault();
c.addEventListener('mousedown',e=>{if(!locked)return;let r=ray();if(!r)return;if(e.button===0&&r.hit){set(...r.hit,0);build()}if(e.button===2&&r.prev&&get(...r.prev)===0&&!inside(...r.prev)){set(...r.prev,placeTypes[selected]);build()}});
addEventListener('wheel',e=>{selected=(selected+(e.deltaY>0?1:types.length-1))%types.length;hud()},{passive:true});document.addEventListener('pointerlockchange',()=>{locked=document.pointerLockElement===c;menu.classList.toggle('hide',locked)});play.onclick=()=>c.requestPointerLock();addEventListener('mousemove',e=>{if(locked){yaw+=e.movementX*.0025;pitch=Math.max(-1.35,Math.min(1.35,pitch-e.movementY*.0025))}});
function frame(t){let d=Math.min(.04,(t-last)/1000)||0;last=t;flowTimer+=d;if(flowTimer>.45){flowTimer=0;flowWater()}if(locked){let f=(keys.z||keys.w?1:0)-(keys.s?1:0),s=(keys.d?1:0)-(keys.q||keys.a?1:0),l=Math.hypot(f,s)||1,sp=(keys.shift?10:5)*d,nx=px+(Math.sin(yaw)*f+Math.cos(yaw)*s)/l*sp,nz=pz+(-Math.cos(yaw)*f+Math.sin(yaw)*s)/l*sp;if(!collide(nx,py,pz))px=nx;if(!collide(px,py,nz))pz=nz;if(fly){let up=(keys[' ']?1:0)-(keys.control||keys.c?1:0);py+=up*sp;vy=0;onGround=false}else{vy-=18*d;if(keys[' ']&&onGround){vy=7;onGround=false}let ny=py+vy*d;if(vy>0&&collide(px,ny,pz)){vy=0;ny=py}if(vy<=0&&feet(px,ny,pz)){ny=Math.floor(ny-1.72)+2.7;vy=0;onGround=true}else onGround=false;py=ny}if(py<-12){let h=heightAt(N/2,N/2);px=N/2+.5;py=h+4;pz=N/2+.5;vy=0}hud()}
let r=Math.min(devicePixelRatio||1,2),w=innerWidth,h=innerHeight;if(c.width!==w*r||c.height!==h*r){c.width=w*r;c.height=h*r;c.style.width=w+'px';c.style.height=h+'px';gl.viewport(0,0,c.width,c.height)}gl.clearColor(.50,.73,.92,1);gl.clear(gl.COLOR_BUFFER_BIT|gl.DEPTH_BUFFER_BIT);gl.enable(gl.DEPTH_TEST);gl.bindBuffer(gl.ARRAY_BUFFER,buf);gl.vertexAttribPointer(ap,3,gl.FLOAT,false,24,0);gl.vertexAttribPointer(ac,3,gl.FLOAT,false,24,12);gl.enableVertexAttribArray(ap);gl.enableVertexAttribArray(ac);gl.uniform3f(uc,px,py,pz);gl.uniform1f(uyaw,yaw);gl.uniform1f(upitch,pitch);gl.uniform1f(uaspect,c.width/c.height);gl.drawArrays(gl.TRIANGLES,0,mesh);requestAnimationFrame(frame)}
hud();requestAnimationFrame(frame);
})();
