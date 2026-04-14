shader_type spatial;
uniform sampler3D Noise1 : repeat_enable;
uniform sampler3D Noise2 : repeat_enable;

void vertex() {
	vec3 sample_pos1 = TIME * 0.1 + vec3(VERTEX.x, 0, VERTEX.z) * 0.05;
	vec3 sample_pos2 = -TIME * 0.1 + vec3(VERTEX.x, 0, VERTEX.z) * 0.05;

	float sample1 = texture(Noise1, sample_pos1).r;
	float sample2 = texture(Noise2, sample_pos2).r;
	VERTEX.y += sample1 + sample2;
	
	float sample_px = texture(Noise1, sample_pos1 + vec3( 0.05, 0, 0)).r;
	float sample_nx = texture(Noise1, sample_pos1 + vec3(-0.05, 0, 0)).r;
	float sample_pz = texture(Noise1, sample_pos1 + vec3(0, 0,  0.05)).r;
	float sample_nz = texture(Noise1, sample_pos1 + vec3(0, 0, -0.05)).r;

	NORMAL = vec3((sample_nx - sample_px) / 0.9, 1.0, (sample_nz - sample_pz) / 2.0);
}

void fragment() {
	// Called for every pixel the material is visible on.
	ALBEDO.r = 0.0;
	ALBEDO.g = 0.0;
	ALBEDO.b = 0.1;
}

//void light() {
//	// Called for every pixel for every light affecting the material.
//	// Uncomment to replace the default light processing function with this one.
//}
