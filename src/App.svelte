<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri"
  import { onMount } from "svelte";
  import { createScene } from "./scene";
  import * as THREE from 'three';
  import * as BufferGeometryUtils from 'three/addons/utils/BufferGeometryUtils.js';

  let meshes = [],
      hex = [],
      show_vertices = true,
      show_faces = false,
      el, 
      scene,
      current_mesh,
      vertice_count,
      face_count,
      resource_location,
      geometry,
      vertice_mesh,
      rendered_mesh;

  onMount(async () => {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    meshes = await invoke("get_meshes")
    scene = createScene(el);

    const directionalLight = new THREE.DirectionalLight( 0xffffff, 1 );
    scene.add( directionalLight );
  });

  $: {
    if(vertice_mesh && rendered_mesh) {
      console.log('showing', show_vertices, show_faces);
      vertice_mesh.visible = show_vertices;
      rendered_mesh.visible = show_faces;
    }
  }

  const loadMesh = async (name, path) => {
    while(scene.children.length > 0){ 
      scene.remove(scene.children[0]); 
    }

    let meshes = await invoke("load_file", { filePath: path });
    current_mesh = name;
    resource_location = path;
    hex = meshes.map(data => data.toString(16).padStart(2, '0'));
  }

  const renderMesh = async () => {
    let vertices = await getVertices();
    await getFaces(vertices);
  }

  const getVertices = async () => {
    geometry = new THREE.BufferGeometry();
    let sprite = new THREE.TextureLoader().load('./src/assets/textures/disc.png');
    let vertices = await invoke("get_vertices", { padding: 20, padInterval: 0 });

    let verticesArray = new Float32Array(vertices.length * 3); // each vertex has x, y, z
    vertices.forEach((vertex, i) => {
        verticesArray[i * 3] = vertex[0];
        verticesArray[i * 3 + 1] = vertex[1];
        verticesArray[i * 3 + 2] = vertex[2];
    });

    vertice_count = vertices.length;

    geometry.setAttribute('position', new THREE.BufferAttribute(verticesArray, 3));

    // Use PointsMaterial for point cloud materials
    let material = new THREE.PointsMaterial({
        size: 3,
        sizeAttenuation: false,
        map: sprite,
        transparent: false,
    });
    material.color.setHSL(1.0, 1.0, 0.5);

    // Use Points instead of the deprecated PointCloud
    let particles = new THREE.Points(geometry, material);

    // Add particles to the scene
    scene.add(particles);

    vertice_mesh = particles;

    return verticesArray;
  }

  const getFaces = async (vertices = []) => {
    let faces = await invoke("get_faces", { padding: 0, padInterval: 0 });
    face_count = faces.length;
    let geo = new THREE.BufferGeometry();

    let indices = new Uint16Array(faces.length * 3);

    for(let i = 0; i < faces.length; i++) {
      indices[i * 3] = faces[i][0];
      indices[i * 3 + 1] = faces[i][1];
      indices[i * 3 + 2] = faces[i][2];
    }

    let facesArray = new Uint16Array(faces.length * 3); // each face has 3 vertices
    faces.forEach((face, i) => {
        facesArray[i * 3] = face[0];
        facesArray[i * 3 + 1] = face[1];
        facesArray[i * 3 + 2] = face[2];
    });

    //geo.setIndex(facesArray);
    geo.setIndex(new THREE.BufferAttribute(facesArray, 3));
    geo.setAttribute('position', new THREE.BufferAttribute(vertices, 3));
    geo.computeVertexNormals();

    geo = BufferGeometryUtils.toTrianglesDrawMode(geo, THREE.TriangleStripDrawMode);

    const material = new THREE.MeshBasicMaterial({ color: 0x808080 });
    material.side = THREE.DoubleSide;
    const mesh = new THREE.Mesh(geo, material);
    scene.add(mesh);

    console.log('vertices', vertices);
    console.log('faces', facesArray);

    rendered_mesh = mesh;
  }
</script>

<main class="container" style="padding: 0; margin: 0;">
  <div style="position: absolute; display: flex; top: 5px; left: 5px; flex-direction: column;">
    <div style="display: flex; flex-direction: row; z-index: 999;">
      <button style="margin-right: 10px; background-color: dodgerblue;">Meshes</button>
      <button style="margin-right: 10px;">Textures</button>
      {#if current_mesh}
        <p style="background-color: rgba(130, 130, 130, 0.25); margin-right: 10px; border-radius: 10px; padding-left: 10px; padding-right: 10px; padding-top: 5px; paddding-bottom: 5px; color: white;">{current_mesh}</p>
        <p style="background-color: rgba(130, 130, 130, 0.25); margin-right: 10px; border-radius: 10px; padding-left: 10px; padding-right: 10px; padding-top: 5px; paddding-bottom: 5px; color: white;">{resource_location}</p>
        <p style="background-color: rgba(130, 130, 130, 0.25); margin-right: 10px; border-radius: 10px; padding-left: 10px; padding-right: 10px; padding-top: 5px; paddding-bottom: 5px; color: white;">Vertex Count: {vertice_count}</p>
        <p style="background-color: rgba(130, 130, 130, 0.25); margin-right: 10px; border-radius: 10px; padding-left: 10px; padding-right: 10px; padding-top: 5px; paddding-bottom: 5px; color: white;">Face Count: {face_count}</p>
        <p style="background-color: rgba(130, 130, 130, 0.25); margin-right: 10px; border-radius: 10px; padding-left: 10px; padding-right: 10px; padding-top: 5px; paddding-bottom: 5px; color: white;">
          <input type="checkbox" bind:checked={show_vertices} /> Show Vertices
        </p>
        <p style="background-color: rgba(130, 130, 130, 0.25); margin-right: 10px; border-radius: 10px; padding-left: 10px; padding-right: 10px; padding-top: 5px; paddding-bottom: 5px; color: white;">
          <input type="checkbox" bind:checked={show_faces} /> Show Faces
        </p>
      {/if}
    </div>
    <div style="margin-top: 10px; max-height: 90vh; max-width: 200px; overflow: auto; background-color: rgba(130, 130, 130, 0.25); border-radius: 10px; -ms-overflow-style: none; scrollbar-width: none; z-index: 999;">
      <ul>
        {#each meshes as mesh}
          <li>
            <a href="#" on:click|preventDefault={async () => { await loadMesh(mesh.name, mesh.path); await renderMesh(); }}>
              {mesh.name.replace(/\(.*\)\.msh/g, '')}
            </a>
          </li>
        {/each}
      </ul>
    </div>
  </div>
  
  <div style="display: flex; flex-direction: row;">
    <div class="column" style="width: 100%; display: flex; flex-direction: row; z-index: 10;">
      <canvas style="margin: 0 auto;" bind:this={el}></canvas>
    </div>
  </div>


</main>

<style>
  ul {
    list-style-type: none;
    text-align: left;
  }
</style>