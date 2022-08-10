#version 130

uniform sampler2D colorTexture;
uniform sampler2D depthTexture;
uniform vec2 canvasResolution;

in vec2 vTexCoords;

mat3 sx = mat3(
	1.0, 2.0, 1.0,
	0.0, 0.0, 0.0,
	-1.0, -2.0, -1.0
);

mat3 sy = mat3(
	1.0, 0.0, -1.0,
	2.0, 0.0, -2.0,
	1.0, 0.0, -1.0
);

float linearizeDepth(float z)
{
	float n = 0.1f;
	float f = 500.0f;
	return (2.0f * n) / (f + n - z * (f - n));
}

vec3 sobelEdgeColorOnColor()
{
	vec3 diffuse = texture(colorTexture, vTexCoords).rgb;
	mat3 I;
	for (int i = 0; i < 3; ++i)
	{
		for (int j = 0; j < 3; ++j)
		{
			vec3 color = texelFetch(colorTexture, ivec2(gl_FragCoord) + ivec2(i - 1.0f, j - 1.0f), 0).rgb;
			I[i][j] = length(color);
		}
	}

	float gx = dot(sx[0], I[0]) + dot(sx[1], I[1]) + dot(sx[2], I[2]);
	float gy = dot(sy[0], I[0]) + dot(sy[1], I[1]) + dot(sy[2], I[2]);

	float g = sqrt(pow(gx, 2.0) + pow(gy, 2.0));
	return vec3(g);
}

vec3 sobelEdgeColorOnDepth()
{
	vec3 diffuse = texture(depthTexture, vTexCoords).rgb;
	mat3 I;
	for (int i = 0; i < 3; ++i)
	{
		for (int j = 0; j < 3; ++j)
		{
			float z = texelFetch(depthTexture, ivec2(gl_FragCoord) + ivec2(i - 1.0f, j - 1.0f), 0).r;
			z = linearizeDepth(z);
			I[i][j] = z;
		}
	}

	float gx = dot(sx[0], I[0]) + dot(sx[1], I[1]) + dot(sx[2], I[2]);
	float gy = dot(sy[0], I[0]) + dot(sy[1], I[1]) + dot(sy[2], I[2]);

	float g = sqrt(pow(gx, 2.0) + pow(gy, 2.0));
	return vec3(g);
}

void main()
{
	float depth = linearizeDepth(texture(depthTexture, vTexCoords).x);
	gl_FragDepth = depth;

	vec3 color = texture(colorTexture, vTexCoords).rgb;

	// color
//	gl_FragColor = texture(colorTexture, vTexCoords);

	// color with black edge
//	gl_FragColor = vec4(color - sobelEdgeColorOnDepth(), 1.0f);

	// only black edges
//	gl_FragColor = vec4(vec3(1.0f, 1.0f, 1.0f) - sobelEdgeColorOnDepth(), 1.0);

	// only white edges
//	gl_FragColor = vec4(sobelEdgeColorOnDepth(), 1.0);

	// depth map
	gl_FragColor = vec4(depth, depth, depth, 1.0f);
}
