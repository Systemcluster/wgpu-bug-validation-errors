#pragma shader_stage(vertex)

cbuffer Camera : register(b0) {
	float4x4 projection;
}

struct Input {
	uint vertexID : SV_VERTEXID;

    float4 position;
	float4 rotation;
	float4 size;

    float2 texture_position;
    float2 texture_size;
};

struct Output {
	float4 position : SV_POSITION;
	float2 uv : TEXCOORD0;
};

float2 rotate_point(float pointX, float pointY, float originX, float originY, float angle) {
    return float2(
        cos(angle) * (pointX-originX) - sin(angle) * (pointY-originY) + originX,
        sin(angle) * (pointX-originX) + cos(angle) * (pointY-originY) + originY
	);
}

Output main(Input input) {
	Output o;

    float tpx = input.texture_position.x;
    float tpy = input.texture_position.y;
    float tsx = input.texture_size.x;
    float tsy = input.texture_size.y;

    float2 utl = float2(tpx,       tpy      );
	float2 ubl = float2(tpx,       tpy + tsy);
	float2 ubr = float2(tpx + tsx, tpy + tsy);
	float2 utr = float2(tpx + tsx, tpy      );

	float2 points[6] = {
		utl,
		ubl,
		ubr,
		ubr,
		utr,
		utl
	};

    float2 offset = float2(input.position.x, input.position.y);
    float sx = input.size.x;
    float sy = input.size.y;
    float pz = input.position.z;
	float angle = input.rotation[0];

	float3 tl = float3(rotate_point(-sx, +sy, 0, 0, angle) + offset, pz);
	float3 bl = float3(rotate_point(-sx, -sy, 0, 0, angle) + offset, pz);
	float3 br = float3(rotate_point(+sx, -sy, 0, 0, angle) + offset, pz);
	float3 tr = float3(rotate_point(+sx, +sy, 0, 0, angle) + offset, pz);

	float3 positions[6] = {
		tl,
		bl,
		br,
		br,
		tr,
		tl,
	};

	o.position = float4(positions[input.vertexID], 1.0) * projection;
    o.uv = points[input.vertexID];

    return o;
}
