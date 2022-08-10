out vec3 vNormal;
out vec2 vUvChannels[maxUvChannelCount];

void main()
{
	mat4 boneTransform = 
		bones[int(boneIds[0])] * boneWeights[0] +
		bones[int(boneIds[1])] * boneWeights[1] +
		bones[int(boneIds[2])] * boneWeights[2] +
		bones[int(boneIds[3])] * boneWeights[3];

	for (int i = 0; i < maxUvChannelCount; ++i)
	{
		vUvChannels[i] = uvChannels[i];
	}
	vNormal = normalMatrix * mat3(boneTransform) * normal;
	gl_Position = projectionMatrix * viewMatrix * objectMatrix * boneTransform * vec4(position, 1.0f);
}
