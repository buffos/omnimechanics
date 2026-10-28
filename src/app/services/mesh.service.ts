import { Injectable, signal } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';

export type ElementType =
  | 'Line2'
  | 'Line3'
  | 'Tri3'
  | 'Tri6'
  | 'Quad4'
  | 'Quad8'
  | 'Tet4'
  | 'Tet10'
  | 'Hex8'
  | 'Hex20';

export interface BoundaryElement {
  id: number;
  elem_type: ElementType;
  nodes: number[];
  tag: string;
  normal: [number, number, number];
}

export interface VolumeElement {
  id: number;
  elem_type: ElementType;
  nodes: number[];
}

export interface MeshStats {
  num_nodes: number;
  num_boundary_elements: number;
  num_volume_elements: number;
  dimension: number;
}

export interface MeshData {
  dimension: number;
  nodes: [number, number, number][];
  boundary_elements: BoundaryElement[];
  volume_elements: VolumeElement[];
  stats: MeshStats;
}

export type MeshGeneratorRequest =
  | {
      preset: 'Beam2D';
      params: {
        length: number;
        height: number;
        nx: number;
        ny: number;
        use_triangles: boolean;
      };
    }
  | {
      preset: 'PlateWithHole2D';
      params: {
        width: number;
        height: number;
        radius: number;
        radial_divisions: number;
        tangential_divisions: number;
      };
    }
  | {
      preset: 'Beam3D';
      params: {
        length: number;
        height: number;
        depth: number;
        nx: number;
        ny: number;
        nz: number;
        use_tetrahedra: boolean;
      };
    }
  | {
      preset: 'BlockWithCavity3D';
      params: {
        size: number;
        radius: number;
        divisions: number;
      };
    };

@Injectable({
  providedIn: 'root',
})
export class MeshService {
  readonly currentMesh = signal<MeshData | null>(null);
  readonly isGenerating = signal<boolean>(false);
  readonly generationError = signal<string | null>(null);

  async generateMesh(request: MeshGeneratorRequest): Promise<MeshData> {
    this.isGenerating.set(true);
    this.generationError.set(null);
    try {
      const mesh = await invoke<MeshData>('generate_mesh', { request });
      this.currentMesh.set(mesh);
      return mesh;
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      this.generationError.set(msg);
      throw err;
    } finally {
      this.isGenerating.set(false);
    }
  }
}
