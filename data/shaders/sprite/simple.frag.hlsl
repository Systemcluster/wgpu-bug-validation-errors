#pragma shader_stage(fragment)

Texture2D map : register(t1);
SamplerState sam : register(s2);

struct Input {
	float4 fragCoord : SV_POSITION;
	float2 uv : TEXCOORD0;
};

struct Output {
	float4 color : SV_TARGET0;
};

Output main(Input input) {
	Output o;
    float width;
	float height;
	map.GetDimensions(width, height);
	o.color = map.Sample(sam, float2(input.uv.x, input.uv.y));
	return o;
}
