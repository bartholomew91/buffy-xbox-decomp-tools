<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri"
  import { onMount } from "svelte";
  import { createScene } from "./scene";
  import * as THREE from 'three';
  import { OBJExporter } from 'three/addons/exporters/OBJExporter.js';

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
  });

  $: {
    if(vertice_mesh && rendered_mesh) {
      console.log('showing', show_vertices, show_faces);
      vertice_mesh.visible = show_vertices;
      rendered_mesh.visible = show_faces;
    }
  }

  const addLighting = () => {
      // Directional light for strong directional shading
      const directionalLight = new THREE.DirectionalLight(0xffffff, 0.8);
      directionalLight.position.set(5, 10, 7.5); // Adjust the position to suit the scene
      scene.add(directionalLight);

      // Ambient light for subtle overall lighting
      const ambientLight = new THREE.AmbientLight(0x404040, 5); // Soft white light
      scene.add(ambientLight);

      // Hemisphere light for soft sky-like lighting
      const hemisphereLight = new THREE.HemisphereLight(0xffffbb, 0x080820, 5);
      scene.add(hemisphereLight);

      // Optionally, you can add more lights like a PointLight or another DirectionalLight
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
    let verticesArray = await getVertices();
    await getFaces(verticesArray); // Ensure you pass verticesArray from getVertices
}

const getVertices = async () => {
    geometry = new THREE.BufferGeometry();
    
    let vertices = await invoke("get_vertices", { padding: 20, padInterval: 0 });

    let verticesArray = new Float32Array(vertices.length * 3); // x, y, z for each vertex
    let normalsArray = new Float32Array(vertices.length * 3);

    vertices.forEach((vertex, i) => {
        verticesArray[i * 3] = vertex[0];
        verticesArray[i * 3 + 1] = vertex[1];
        verticesArray[i * 3 + 2] = vertex[2];
        normalsArray[i * 3] = vertex[3];
        normalsArray[i * 3 + 1] = vertex[4];
        normalsArray[i * 3 + 2] = vertex[5];
    });

    geometry.setAttribute('position', new THREE.BufferAttribute(verticesArray, 3));
    geometry.setAttribute('normal', new THREE.BufferAttribute(normalsArray, 3));

    return verticesArray; // Return the vertices array for use in getFaces
}

const getFaces = async (vertices) => {
    let faces = await invoke("get_faces", { padding: 0, padInterval: 0 });

    let facesArray = new Uint16Array(faces.length * 3); // Indices for each triangle
    faces.forEach((face, i) => {
        facesArray[i * 3] = face[0];
        facesArray[i * 3 + 1] = face[1];
        facesArray[i * 3 + 2] = face[2];
    });

    geometry.setIndex(new THREE.BufferAttribute(facesArray, 1)); // Set face indices

    // Finalize mesh
    const material = new THREE.MeshStandardMaterial({
        color: 0x808080, // Grey color
        metalness: 0.5, // For shiny surface, only in MeshStandardMaterial
        roughness: 0.7, // Roughness controls smoothness of surface, only in MeshStandardMaterial
        side: THREE.DoubleSide, // Ensures both sides of the mesh are visible
    });
    const mesh = new THREE.Mesh(geometry, material);
    
    scene.add(mesh);

    // Add lighting
    addLighting();

    console.log('vertices', vertices);
    console.log('faces', facesArray);
}

const exportOBJ = () => {
  const exporter = new OBJExporter();
  const objData = exporter.parse(scene);
  const blob = new Blob([objData], { type: 'text/plain' });

  const link = document.createElement('a');
  link.href = URL.createObjectURL(blob);
  link.download = current_mesh.replace('msh', 'obj');
  link.click();
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
        <!-- <p style="background-color: rgba(130, 130, 130, 0.25); margin-right: 10px; border-radius: 10px; padding-left: 10px; padding-right: 10px; padding-top: 5px; paddding-bottom: 5px; color: white;">
          <input type="checkbox" bind:checked={show_vertices} /> Show Vertices
        </p> -->
        <!-- <p style="background-color: rgba(130, 130, 130, 0.25); margin-right: 10px; border-radius: 10px; padding-left: 10px; padding-right: 10px; padding-top: 5px; paddding-bottom: 5px; color: white;">
          <input type="checkbox" bind:checked={show_faces} /> Show Faces
        </p> -->
        <p>
          <button on:click|preventDefault={exportOBJ}>Export OBJ</button>
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