struct Ray {
  vec3 Origin;
  vec3 Direction;
};

vec3 ray_at(Ray ray, float t) {
  return ray.Origin + t * ray.Direction;
}
