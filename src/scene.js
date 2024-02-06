import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';

const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, 1, 0.1, 2000);

let renderer;
let controls;

camera.position.z = 50;
camera.position.x = 0;



const animate = () => {
    requestAnimationFrame(animate);    
    controls.update();
    renderer.render(scene, camera);
};

const resize = () => {
    camera.aspect = 1920 / 1080;
    camera.updateProjectionMatrix();
    renderer.setSize(1920, 1080);
};

export const createScene = (el) => {
    renderer = new THREE.WebGLRenderer({ antialias: true, canvas: el });
    controls = new OrbitControls( camera, el );
    controls.update();
    resize();
    animate();
    return scene;
}

window.addEventListener('resize', resize);